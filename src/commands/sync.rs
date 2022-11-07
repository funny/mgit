use super::{
    clean, cmp_local_remote, display_path, execute_cmd, find_remote_name_by_url, load_config,
    ResetType, StashMode, TomlRepo,
};
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

pub fn exec(
    path: Option<String>,
    config: Option<PathBuf>,
    stash_mode: StashMode,
    num_threads: usize,
    silent: bool,
) {
    let cwd = env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.clone().or(cwd_str).unwrap();

    // starting sync repos
    println!("sync repos in {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // if directory doesn't exist, use --hard
    let stash_mode = match input_path.is_dir() {
        true => stash_mode,
        false => StashMode::Hard,
    };

    // set config file path
    let config_file = match config.clone() {
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
        let default_branch = toml_config.default_branch;

        // remove unused repositories when use '--config' option
        // also if input_path not exists, skip this process
        if stash_mode == StashMode::Hard && input_path.is_dir() {
            clean::exec(path, config);
        }

        // handle sync
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

                        // execute command according each repo status
                        let execute_result = execute_sync_with_progress(
                            input_path,
                            toml_repo,
                            &stash_mode,
                            &default_branch,
                            &prefix,
                            &progress_bar,
                        );

                        // handle result
                        let result = match execute_result {
                            Ok(_) => {
                                let mut message = format!(
                                    "{} {} {}",
                                    "√".bold().green(),
                                    &prefix,
                                    display_path(toml_repo.local.as_ref().unwrap())
                                        .bold()
                                        .magenta()
                                );

                                // if not silent, show compare stat betweent local and remote
                                if silent == false {
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

                                // Truncates message string to a certain number of characters.
                                let truncated_message = truncate_str(&message, 70, "...");
                                // show meeshage in progress bar
                                progress_bar.finish_with_message(format!("{}", truncated_message));
                                Ok(())
                            }
                            Err(e) => {
                                progress_bar.finish_with_message(format!(
                                    "{} {} {}",
                                    "x".bold().red(),
                                    &prefix,
                                    display_path(toml_repo.local.as_ref().unwrap())
                                        .bold()
                                        .magenta(),
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
                        display_path(toml_repo.local.as_ref().unwrap())
                            .bold()
                            .magenta()
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
    stash_mode: &StashMode,
    default_branch: &Option<String>,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = &input_path.join(rel_path);

    // make repo directory and skip clone the repository
    std::fs::create_dir_all(full_path)
        .with_context(|| format!("create dir {} failed.", full_path.to_str().unwrap()))?;

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

    match stash_mode {
        StashMode::Normal => {
            // reset --soft
            execute_reset_with_progress(
                input_path,
                toml_repo,
                ResetType::Soft,
                prefix,
                progress_bar,
            )
        }
        StashMode::Stash => {
            // stash with `--stash` option, maybe return error firstly without any commit
            let stash_result =
                execute_stash_with_progress(input_path, toml_repo, prefix, progress_bar);
            let stash_message = stash_result.unwrap_or("stash failed.".to_string());

            // reset --mixed
            execute_reset_with_progress(
                input_path,
                toml_repo,
                ResetType::Mixed,
                prefix,
                progress_bar,
            )
            .with_context(|| stash_message.clone())
            .or_else(|e| {
                // if reset failed, pop stash if stash something this time
                if stash_message.contains("WIP") {
                    let _ = execute_stash_pop_with_progress(
                        input_path,
                        toml_repo,
                        prefix,
                        progress_bar,
                    );
                }
                Err(e)
            })
        }
        StashMode::Hard => {
            // clean
            execute_clean_with_progress(input_path, toml_repo, prefix, progress_bar)?;

            // reset
            execute_reset_with_progress(
                input_path,
                toml_repo,
                ResetType::Hard,
                prefix,
                progress_bar,
            )
        } // stash without `--force` option, maybe return error first without any commit
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
//     let args = [
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
    let full_path = input_path.join(rel_path);

    // try open git repo
    let repo = Repository::open(&full_path)?;
    // get remote name from url
    let remote_url = toml_repo
        .remote
        .as_ref()
        .with_context(|| "remote url is null.")?;
    let remote_name = find_remote_name_by_url(&repo, remote_url)?;

    let args = [
        "fetch",
        &remote_name,
        "--prune",
        "--recurse-submodules=on-demand",
        "--progress",
    ];

    let mut command = Command::new("git");
    let full_command = command.args(args).current_dir(full_path);

    execute_with_progress(rel_path, full_command, prefix, progress_bar)
}

fn execute_init_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    progress_bar.set_message(format!(
        "{:>9} {} : initializing...",
        prefix,
        display_path(rel_path).bold().magenta()
    ));

    let args = ["init"];

    match execute_cmd(&full_path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}

fn execute_add_remote_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    progress_bar.set_message(format!(
        "{:>9} {} : addding remote...",
        prefix,
        display_path(rel_path).bold().magenta()
    ));

    // git remote add origin {url}
    let args = [
        "remote",
        "add",
        "origin",
        toml_repo.remote.as_ref().unwrap(),
    ];

    match execute_cmd(&full_path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}

fn execute_clean_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    progress_bar.set_message(format!(
        "{:>9} {} : cleaning...",
        prefix,
        display_path(rel_path).bold().magenta()
    ));

    let args = ["clean", "-fd"];

    match execute_cmd(&full_path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}

fn execute_reset_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    reset_type: ResetType,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    progress_bar.set_message(format!(
        "{:>9} {} : resetting...",
        prefix,
        display_path(rel_path).bold().magenta()
    ));

    // priority: commit/tag/branch(default-branch)
    let remote_ref = {
        if let Some(commit) = &toml_repo.commit {
            commit.to_string()
        } else if let Some(tag) = &toml_repo.tag {
            tag.to_string()
        } else if let Some(branch) = &toml_repo.branch {
            "origin/".to_string() + &branch.to_string()
        } else {
            String::new()
        }
    };

    let reset_type = match reset_type {
        ResetType::Soft => "--soft",
        ResetType::Mixed => "--mixed",
        ResetType::Hard => "--hard",
    };
    let args = ["reset", reset_type, &remote_ref];

    match execute_cmd(&full_path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}

fn execute_stash_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> Result<String, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    progress_bar.set_message(format!(
        "{:>9} {} : stashing...",
        prefix,
        display_path(rel_path).bold().magenta()
    ));

    let args = ["stash", "--include-untracked"];

    execute_cmd(&full_path, "git", &args)
}

fn execute_stash_pop_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> Result<String, anyhow::Error> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    progress_bar.set_message(format!(
        "{:>9} {} : pop stash...",
        prefix,
        display_path(rel_path).bold().magenta()
    ));

    let args = ["stash", "pop"];

    execute_cmd(&full_path, "git", &args)
}

fn execute_with_progress(
    rel_path: &String,
    command: &mut Command,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let mut spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Error starting command {:?}", command))?;

    let mut last_line = format!(
        "{:>9} {}: running...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
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
                display_path(rel_path).bold().magenta(),
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
            "Git exited with code {}: {}. With command : {:?}.",
            exit_code.code().unwrap(),
            last_line.trim(),
            command
        ));
    }

    Ok(())
}
