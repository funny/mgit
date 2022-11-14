use super::{
    cmp_local_remote, display_path, execute_cmd_with_progress, fmt_msg_spinner, load_config,
    TomlRepo,
};
use atomic_counter::{AtomicCounter, RelaxedCounter};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::{iter::ParallelIterator, prelude::IntoParallelRefIterator};
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

pub fn exec(path: Option<String>, config: Option<PathBuf>, num_threads: usize, silent: bool) {
    let cwd = env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.or(cwd_str).unwrap();

    // starting fetch repos
    println!("fetch repos in {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // check if input is a valid directory
    if !input_path.is_dir() {
        println!("Directory {} not found!", input.bold().magenta());
        return;
    }

    // set config file path
    let config_file = match config {
        Some(r) => r,
        _ => input_path.join(".gitrepos"),
    };

    // check if .gitrepos exists
    if !config_file.is_file() {
        println!(
            "{} not found, try {} instead!",
            ".gitrepos".bold().magenta(),
            "init".bold().magenta()
        );
        return;
    }

    // load .gitrepos
    if let Some(toml_config) = load_config(&config_file) {
        let default_branch = toml_config.default_branch;

        // handle fetch
        if let Some(toml_repos) = toml_config.repos {
            let repos_count = toml_repos.len();

            // multi_progress manages multiple progress bars from different threads
            // use Arc to share the MultiProgress across more than 1 thread
            let multi_progress = Arc::new(MultiProgress::new());
            // create total progress bar and set progress style
            let total_bar = multi_progress.add(ProgressBar::new(repos_count as u64));
            total_bar.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}] {percent}% [{bar:30.green/white}] {pos}/{len}",
                )
                .unwrap()
                .progress_chars("=>-"),
            );
            total_bar.enable_steady_tick(std::time::Duration::from_millis(500));

            // user a counter
            let counter = RelaxedCounter::new(1);

            // Clone Arc<MultiProgress> and spawn a thread.
            // need to do this in a thread as the `.map()` we do below also blocks.
            let multi_progress_wait = multi_progress.clone();

            // create thread pool, and set the number of thread to use by using `.num_threads(count)`
            let thread_pool = match rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            };

            // pool.install means that `.par_iter()` will use the thread pool we've built above.
            let errors: Vec<(&TomlRepo, anyhow::Error)> = thread_pool.install(|| {
                let res = toml_repos
                    .par_iter()
                    .map(|toml_repo| {
                        let idx = counter.inc();
                        let prefix = format!("[{:02}/{:02}]", idx, repos_count);

                        // create progress bar for each repo
                        let progress_bar =
                            multi_progress_wait.insert(idx, ProgressBar::new_spinner());
                        progress_bar.set_style(
                            ProgressStyle::with_template("{spinner:.green.dim.bold} {msg} ")
                                .unwrap()
                                .tick_chars("/-\\| "),
                        );
                        progress_bar.enable_steady_tick(std::time::Duration::from_millis(500));
                        let message = format!("{:>9} waiting...", &prefix);
                        progress_bar.set_message(fmt_msg_spinner(&message));

                        // execute fetch command with progress
                        let execute_result = execute_fetch_with_progress(
                            input_path,
                            toml_repo,
                            &prefix,
                            &progress_bar,
                        );

                        // handle result
                        let result = match execute_result {
                            Ok(_) => {
                                // check if show compare message betweent local and remote
                                let mut message = format!(
                                    "{} {} {}",
                                    "√".bold().green(),
                                    &prefix,
                                    display_path(toml_repo.local.as_ref().unwrap())
                                        .bold()
                                        .magenta()
                                );

                                // if not silent, show compare stat betweent local and remote
                                if !silent {
                                    // get compare stat betwwen local and specified commit/tag/branch/
                                    let cmp_msg = match cmp_local_remote(
                                        input_path,
                                        toml_repo,
                                        &default_branch,
                                    ) {
                                        Ok(r) => r.unwrap(),
                                        _ => String::new(),
                                    };

                                    message = format!("{}: {}", message, &cmp_msg)
                                };

                                // show meeshage in progress bar
                                progress_bar.finish_with_message(fmt_msg_spinner(&message));
                                Ok(())
                            }
                            Err(e) => {
                                let message = format!(
                                    "{} {} {}",
                                    "x".bold().red(),
                                    &prefix,
                                    display_path(toml_repo.local.as_ref().unwrap())
                                        .bold()
                                        .magenta(),
                                );

                                // show meeshage in progress bar
                                progress_bar.finish_with_message(fmt_msg_spinner(&message));
                                Err((toml_repo, e))
                            }
                        };

                        // update total progress bar
                        total_bar.inc(1);

                        result
                    })
                    // catch erroring repo
                    .filter_map(Result::err)
                    // collect the results into Vec
                    .collect();

                total_bar.finish();

                res
            });

            // show sync stat
            println!("\n");
            if errors.is_empty() {
                println!("fetch finished! 0 error(s).");
            } else {
                println!(
                    "fetch finished! {} error(s).",
                    errors.len().to_string().bold().red()
                );
            }

            // show errors
            if !errors.is_empty() {
                println!("");
                println!("Errors:",);
                for (toml_repo, error) in errors {
                    let mut err_msg = String::new();
                    for e in error.chain() {
                        err_msg += &e.to_string();
                    }
                    eprintln!(
                        "{} {}",
                        display_path(toml_repo.local.as_ref().unwrap())
                            .bold()
                            .magenta(),
                        err_msg.trim().red()
                    );
                    println!("");
                }
            }
        }
    }
}

fn execute_fetch_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    // get remote name from url
    let remote_name = toml_repo.get_remote_name(full_path.as_path())?;

    let args = [
        "fetch",
        &remote_name,
        "--prune",
        "--recurse-submodules=on-demand",
        "--progress",
    ];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(full_path);

    execute_cmd_with_progress(rel_path, full_command, prefix, progress_bar)
}
