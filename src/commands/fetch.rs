use super::{find_remote_name_by_url, load_config, TomlRepo};
use anyhow::Context;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use console::{strip_ansi_codes, truncate_str};
use git2::Repository;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::{iter::ParallelIterator, prelude::IntoParallelRefIterator};
use std::{
    env,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    process::Stdio,
    sync::Arc,
};

pub fn exec(path: Option<String>, config: Option<PathBuf>, num_threads: usize) {
    let cwd = env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.or(cwd_str).unwrap();

    // starting fetch repos
    println!("fetch repos in {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // check if input is a valid directory
    if input_path.is_dir() == false {
        println!("Directory {} not found!", input.bold().magenta());
        return;
    }

    // set config file path
    let config_file = match config {
        Some(r) => r,
        _ => input_path.join(".gitrepos"),
    };

    // check if .gitrepos exists
    if config_file.is_file() == false {
        println!(
            "{} not found, try {} instead!",
            ".gitrepos".bold().magenta(),
            "init".bold().magenta()
        );
        return;
    }

    // load .gitrepos
    if let Some(toml_config) = load_config(&config_file) {
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
                .progress_chars("=>·"),
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
                        progress_bar.set_message(format!("{:>9} waiting...", &prefix));

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
                                progress_bar.finish_with_message(format!(
                                    "{} {} {}",
                                    "√".bold().green(),
                                    &prefix,
                                    toml_repo.local.as_ref().unwrap().bold().magenta()
                                ));
                                Ok(())
                            }
                            Err(e) => {
                                progress_bar.finish_with_message(format!(
                                    "{} {} {}",
                                    "x".bold().red(),
                                    &prefix,
                                    toml_repo.local.as_ref().unwrap().bold().magenta()
                                ));
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

            println!(
                "{} repositories fecth successfully.",
                repos_count - errors.len()
            );

            // print out each repo that failed to fetch
            if !errors.is_empty() {
                eprintln!("{} repositories failed.", errors.len());
                eprintln!("");

                for (toml_repo, error) in errors {
                    eprintln!(
                        "{} errors:",
                        toml_repo.local.as_ref().unwrap().bold().magenta()
                    );
                    error
                        .chain()
                        .for_each(|cause| eprintln!("  {}", cause.bold().red()));
                    eprintln!("");
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
    let full_path = &input_path.join(rel_path);

    // try open git repo
    let repo = Repository::open(full_path)?;
    // get remote name from url
    let remote_url = toml_repo
        .remote
        .as_ref()
        .with_context(|| "remote url is null.")?;
    let remote_name = find_remote_name_by_url(&repo, remote_url)?;

    let args = vec![
        "fetch",
        &remote_name,
        "--prune",
        "--recurse-submodules=on-demand",
        "--progress",
    ];

    let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();

    let local_path = toml_repo.local.as_ref().unwrap();

    let mut command = Command::new("git".to_string());
    let full_command = command.args(args).current_dir(input_path.join(local_path));

    let mut spawned = full_command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Error starting command {:?}", full_command))?;

    let mut last_line = format!("{:>9} {}: running...", prefix, local_path.bold().magenta());
    progress_bar.set_message(last_line.clone());

    // get message from stderr with "--progress" option
    if let Some(ref mut stderr) = spawned.stderr {
        let lines = BufReader::new(stderr).split(b'\r');
        for line in lines {
            let output = line.unwrap();
            if output.is_empty() {
                continue;
            }
            let line = std::str::from_utf8(&output).unwrap();
            let plain_line = strip_ansi_codes(line).replace('\n', " ");
            let full_line = format!(
                "{:>9} {}: {}",
                prefix,
                rel_path.bold().magenta(),
                plain_line.trim()
            );
            let truncated_line = truncate_str(&full_line, 70, "...");
            progress_bar.set_message(format!("{}", truncated_line));
            last_line = plain_line;
        }
    }

    let exit_code = spawned
        .wait()
        .context("Error waiting for process to finish")?;

    if !exit_code.success() {
        return Err(anyhow::anyhow!(
            "Git exited with code {}: {}",
            exit_code.code().unwrap(),
            last_line
        ));
    }

    Ok(())
}
