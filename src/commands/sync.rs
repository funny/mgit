use super::{load_config, TomlRepo};
use anyhow::Context;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use console::{strip_ansi_codes, truncate_str};
use git2::Repository;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::{iter::ParallelIterator, prelude::IntoParallelRefIterator};
use std::{
    env,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    process::Stdio,
    sync::Arc,
    thread,
    thread::JoinHandle,
};

pub fn exec(path: Option<String>, config: Option<PathBuf>, force: bool) {
    let cwd = env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.or(cwd_str).unwrap();

    // starting sync repos
    println!("sync repos in {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // set config file path
    let config_file = match config {
        Some(r) => cwd.join(r),
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
        let default_branch = toml_config.default_branch;

        // handle sync
        if let Some(toml_repos) = toml_config.repos {
            let repos_count = toml_repos.len();

            // multi_progress manages multiple progress bars from different threads
            // use Arc to share the MultiProgress across more than 1 thread
            let multi_progress = Arc::new(MultiProgress::new());
            // create total progress bar and set progress style
            let total_bar = multi_progress.add(ProgressBar::new(repos_count as u64));

            total_bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {percent}% [{bar:30.green/white}] {pos}/{len}")
                    .progress_chars("=>·")
                    .on_finish(ProgressFinish::Abandon),
            );
            total_bar.enable_steady_tick(500);
            // user a counter
            let counter = RelaxedCounter::new(1);

            // Clone Arc<MultiProgress> and spawn a thread.
            // call `.join()` on the `MultiProgress` to updated progress bars.
            // need to do this in a thread as the `.map()` we do below also blocks.
            let multi_progress_wait = multi_progress.clone();
            let thread_handle: JoinHandle<anyhow::Result<()>> = thread::spawn(move || {
                multi_progress_wait.join()?;
                Ok(())
            });

            // create thread pool, and set the number of thread to use by using `.num_threads(count)`
            let thread_pool = match rayon::ThreadPoolBuilder::new().build() {
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
                        let progress_bar = multi_progress.insert(idx, ProgressBar::new_spinner());
                        progress_bar.set_message("waiting...");
                        progress_bar.enable_steady_tick(500);
                        progress_bar.set_style(
                            ProgressStyle::default_spinner()
                                .template("{spinner:.green.dim.bold} {msg} "),
                        );

                        // execute command according each repo status
                        let execute_result = execute_sync_with_progress(
                            input_path,
                            toml_repo,
                            force,
                            &default_branch,
                            &prefix,
                            &progress_bar,
                        );

                        // handle result
                        let result = match execute_result {
                            Ok(_) => {
                                progress_bar.finish_with_message(format!(
                                    "{} {} {}",
                                    "✓".bold().green(),
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

            // wait for the thread to finish
            let _ = thread_handle.join();

            println!(
                "{} repositories sync update successfully.",
                repos_count - errors.len()
            );

            // print out each repo when failed
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
    } else {
        println!(
            "load config file failed. err : {}",
            config_file.to_str().unwrap()
        );
    }
}

fn execute_sync_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    force: bool,
    default_branch: &Option<String>,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = &input_path.join(rel_path);

    // make repo directory and skip clone the repository
    std::fs::create_dir_all(full_path)?;

    let toml_repo = &mut toml_repo.clone();

    // try open git repo
    let repo_result = Repository::open(full_path);

    // if the directory is not a repository
    if let Err(_) = &repo_result {
        // git init when dir exist
        execute_init_with_progress(input_path, toml_repo, prefix, progress_bar)?;
        // git remote add url
        execute_add_remote_with_progress(input_path, toml_repo, prefix, progress_bar)?;
    }

    // use default branch when branch is null
    if None == toml_repo.branch {
        toml_repo.branch = default_branch.to_owned();
    }

    // fetch
    execute_fetch_with_progress(input_path, toml_repo, prefix, progress_bar)?;

    if !force {
        // stash without `--force` option, maybe return error first without any commit
        match execute_stash_with_progress(input_path, toml_repo, prefix, progress_bar).with_context(
            || {
                format!(
                    "With branch : {}.",
                    toml_repo.branch.as_ref().unwrap().to_string()
                )
            },
        ) {
            _ => {
                // reset
                execute_reset_with_progress(input_path, toml_repo, prefix, progress_bar)
            }
        }
    } else {
        // clean
        execute_clean_with_progress(input_path, toml_repo, prefix, progress_bar)?;

        // reset
        execute_reset_with_progress(input_path, toml_repo, prefix, progress_bar)
    }
}

// fn execute_clone_with_progress(
//     input_path: &Path,
//     repo: &TomlRepo,
//     prefix: &str,
//     progress_bar: &ProgressBar,
// ) -> anyhow::Result<()> {
//     let rel_path = repo.local.as_ref().unwrap();
//     let input_path = input_path
//         .join(rel_path.clone())
//         .into_os_string()
//         .into_string()
//         .unwrap()
//         .replace("\\", "/");
//
//     let args = vec![
//         "clone",
//         repo.remote.as_ref().unwrap(),
//         &input_path,
//         "--progress",
//     ];
//     let mut command = Command::new("git");
//     let full_command = command.args(args);
//     execute_with_progress(rel_path, full_command, prefix, progress_bar)
// }

fn execute_fetch_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let input_path = input_path.join(rel_path.clone());

    let args = vec![
        "fetch",
        "--all",
        "--prune",
        "--recurse-submodules=on-demand",
        "--progress",
    ];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(input_path);

    execute_with_progress(rel_path, full_command, prefix, progress_bar)
}

fn execute_init_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let input_path = input_path.join(rel_path);

    let args = vec!["init"];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(input_path);

    execute_with_progress(rel_path, full_command, prefix, progress_bar)
}

fn execute_add_remote_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let input_path = input_path.join(rel_path);

    // git remote add origin {url}
    let args = vec![
        "remote",
        "add",
        "origin",
        toml_repo.remote.as_ref().unwrap(),
    ];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(input_path);

    execute_with_progress(rel_path, full_command, prefix, progress_bar)
}

fn execute_stash_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let input_path = input_path.join(rel_path);

    let args = vec!["stash", "--include-untracked"];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(input_path);

    execute_with_progress(rel_path, full_command, prefix, progress_bar)
}

fn execute_clean_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let input_path = input_path.join(rel_path);

    let args = vec!["clean", "-fd"];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(input_path);

    execute_with_progress(rel_path, full_command, prefix, progress_bar)
}

fn execute_reset_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let input_path = input_path.join(rel_path);

    // branch/default_branch
    let mut repo_head: String;
    repo_head = "origin/".to_string() + toml_repo.branch.as_ref().unwrap();

    // tag
    if let Some(tag) = &toml_repo.tag {
        repo_head = tag.to_string();
    }

    // commit
    if let Some(commit) = &toml_repo.commit {
        repo_head = commit.to_string();
    }

    let args = vec!["reset", "--hard", &repo_head];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(input_path);

    execute_with_progress(rel_path, full_command, prefix, progress_bar)
}

fn execute_with_progress(
    rel_path: &String,
    command: &mut Command,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    progress_bar.set_message(format!(
        "{:9} {} : starting",
        prefix,
        rel_path.bold().magenta()
    ));

    let mut spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Error starting command {:?}", command))?;

    let mut last_line = format!("{:>9} {}: running...", prefix, rel_path.bold().magenta());
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
            let truncated_line = truncate_str(plain_line.trim(), 70, "...");
            progress_bar.set_message(format!(
                "{:>9} {}: {}",
                prefix,
                rel_path.bold().magenta(),
                truncated_line
            ));
            last_line = plain_line;
        }
    }

    let exit_code = spawned
        .wait()
        .context("Error waiting for process to finish")?;

    if !exit_code.success() {
        return Err(anyhow::anyhow!(
            "Git exited with code {}: {}. With command : {:?}.",
            exit_code.code().unwrap(),
            last_line.trim(),
            command
        ));
    }

    Ok(())
}
