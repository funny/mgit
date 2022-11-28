use super::{
    clean, cmp_local_remote, display_path, exclude_ignore, execute_cmd, execute_cmd_with_progress,
    fmt_msg_spinner, get_current_branch, is_remote_ref_valid, is_repository, load_config,
    track::set_tracking_remote_branch, RemoteRef, ResetType, StashMode, TomlRepo,
};
use anyhow::Context;
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

pub fn exec(
    path: Option<String>,
    config: Option<PathBuf>,
    stash_mode: StashMode,
    num_threads: usize,
    silent: bool,
    no_track: bool,
    no_checkout: bool,
    ignore: Option<Vec<String>>,
) {
    let cwd = env::current_dir().unwrap();
    let cwd_str = Some(String::from(cwd.to_string_lossy()));
    let input = path.clone().or(cwd_str).unwrap();

    // starting sync repos
    println!("sync repos in {}", input.bold().magenta());
    let input_path = Path::new(&input);

    // set config file path
    let config_file = match config.clone() {
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

        // remove unused repositories when use '--config' option
        // also if input_path not exists, skip this process
        if stash_mode == StashMode::Hard && input_path.is_dir() {
            clean::exec(path, config);
        }

        // handle sync
        if let Some(toml_repos) = toml_config.repos {
            // ignore specified repositories
            let mut toml_repos = toml_repos;
            exclude_ignore(&mut toml_repos, ignore);

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
            total_bar.tick();

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
            let (succ_repos, error_repos) = thread_pool.install(|| {
                let res: Vec<Result<(&TomlRepo, String), (&TomlRepo, anyhow::Error)>> = toml_repos
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

                        // execute command according each repo status
                        let execute_result = execute_sync_with_progress(
                            input_path,
                            toml_repo,
                            &stash_mode,
                            no_checkout,
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
                                if !silent {
                                    // get compare stat betwwen local and specified commit/tag/branch/
                                    let cmp_msg = match cmp_local_remote(
                                        input_path,
                                        toml_repo,
                                        &default_branch,
                                        false,
                                    ) {
                                        Ok(r) => r.unwrap(),
                                        _ => String::new(),
                                    };

                                    message = format!("{}: {}", message, &cmp_msg)
                                };

                                // show meeshage in progress bar
                                progress_bar.finish_with_message(fmt_msg_spinner(&message));

                                // track remote branch, return track status
                                let mut track_msg = String::new();
                                if !no_track {
                                    if let Ok(res) = set_tracking_remote_branch(
                                        input_path,
                                        &toml_repo,
                                        &default_branch,
                                    ) {
                                        track_msg = res;
                                    }
                                }
                                Ok((toml_repo, track_msg))
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
                    .collect();

                total_bar.finish();

                // collect the err results into Vec
                let mut succ_repos: Vec<(&TomlRepo, String)> = Vec::new();
                // collect the results into Vec
                let mut error_repos: Vec<(&TomlRepo, anyhow::Error)> = Vec::new();
                for r in res {
                    match r {
                        Ok((toml_repo, track_msg)) => succ_repos.push((toml_repo, track_msg)),
                        Err((toml_repo, e)) => error_repos.push((toml_repo, e)),
                    }
                }
                (succ_repos, error_repos)
            });

            // show sync stat
            println!("\n");
            if error_repos.is_empty() {
                println!("sync finished! 0 error(s).");
            } else {
                println!(
                    "sync finished! {} error(s).",
                    error_repos.len().to_string().bold().red()
                );
            }

            // show track status
            if !silent {
                println!("");
                println!("Track status:");
                for (_, track_msg) in &succ_repos {
                    println!("  {}", track_msg);
                }
            }

            // show errors
            if !error_repos.is_empty() {
                println!("");
                println!("Errors:",);
                for (toml_repo, error) in error_repos {
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
    no_checkout: bool,
    default_branch: &Option<String>,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = &input_path.join(rel_path);

    // make repo directory and skip clone the repository
    std::fs::create_dir_all(full_path)
        .with_context(|| format!("create dir {} failed.", full_path.to_str().unwrap()))?;

    let mut toml_repo = toml_repo.to_owned();
    let mut stash_mode = stash_mode.to_owned();
    // if repository not found, create new one
    if is_repository(full_path.as_path()).is_err() {
        // use --hard
        stash_mode = StashMode::Hard;

        // git init when dir exist
        execute_init_with_progress(input_path, &toml_repo, prefix, progress_bar)?;
        // git remote add url
        execute_add_remote_with_progress(input_path, &toml_repo, prefix, progress_bar)?;
    }

    // use default branch when branch is null
    if None == toml_repo.branch {
        toml_repo.branch = default_branch.to_owned();
    }

    // fetch
    execute_fetch_with_progress(input_path, &toml_repo, prefix, progress_bar)?;

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(commit) => commit,
        RemoteRef::Tag(tag) => tag,
        RemoteRef::Branch(branch) => branch,
    };

    // check remote-ref valid
    is_remote_ref_valid(full_path, &remote_ref_str)?;

    match stash_mode {
        StashMode::Normal => {
            // try stash → checkout → reset → stash pop
            if !no_checkout {
                // stash
                let stash_result =
                    execute_stash_with_progress(input_path, &toml_repo, prefix, progress_bar);
                let stash_message = stash_result.unwrap_or("stash failed.".to_string());

                // checkout
                let mut result: Result<(), anyhow::Error>;
                result = execute_checkout_with_progress(
                    input_path,
                    &toml_repo,
                    false,
                    prefix,
                    progress_bar,
                );

                if result.is_ok() {
                    // reset --hard
                    result = execute_reset_with_progress(
                        input_path,
                        &toml_repo,
                        ResetType::Hard,
                        prefix,
                        progress_bar,
                    );
                }

                // stash pop, whether checkout succ or failed, whether reset succ or failed
                if stash_message.contains("WIP") {
                    let _ = execute_stash_pop_with_progress(
                        input_path,
                        &toml_repo,
                        prefix,
                        progress_bar,
                    );
                }
                result
            } else {
                // reset --soft
                execute_reset_with_progress(
                    input_path,
                    &toml_repo,
                    ResetType::Soft,
                    prefix,
                    progress_bar,
                )
            }
        }
        StashMode::Stash => {
            // stash with `--stash` option, maybe return error if need to initial commit
            let stash_result =
                execute_stash_with_progress(input_path, &toml_repo, prefix, progress_bar);

            let stash_message = stash_result.unwrap_or("stash failed.".to_string());

            // checkout
            let mut result: Result<(), anyhow::Error> = Ok(());
            let mut reset_type = ResetType::Mixed;
            if !no_checkout {
                result = execute_checkout_with_progress(
                    input_path,
                    &toml_repo,
                    true,
                    prefix,
                    progress_bar,
                )
                .with_context(|| stash_message.clone());

                reset_type = ResetType::Hard;
            }

            // reset --mixed
            if result.is_ok() {
                result = execute_reset_with_progress(
                    input_path,
                    &toml_repo,
                    reset_type,
                    prefix,
                    progress_bar,
                )
                .with_context(|| stash_message.clone());
            }

            // undo if checkout failed or reset failed
            if let Err(e) = result {
                // if reset failed, pop stash if stash something this time
                if stash_message.contains("WIP") {
                    let _ = execute_stash_pop_with_progress(
                        input_path,
                        &toml_repo,
                        prefix,
                        progress_bar,
                    );
                }
                return Err(e);
            }
            result
        }
        StashMode::Hard => {
            // clean
            execute_clean_with_progress(input_path, &toml_repo, prefix, progress_bar)?;

            // checkout
            if !no_checkout {
                execute_checkout_with_progress(input_path, &toml_repo, true, prefix, progress_bar)?;
            }

            // reset --hard
            execute_reset_with_progress(
                input_path,
                &toml_repo,
                ResetType::Hard,
                prefix,
                progress_bar,
            )
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

fn execute_init_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let message = format!(
        "{:>9} {} : initialize...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

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

    let message = format!(
        "{:>9} {} : add remote...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

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

    let message = format!(
        "{:>9} {} : clean...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

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

    let message = format!(
        "{:>9} {} : reset...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref {
        RemoteRef::Commit(commit) => commit,
        RemoteRef::Tag(tag) => tag,
        RemoteRef::Branch(branch) => branch,
    };

    let reset_type = match reset_type {
        ResetType::Soft => "--soft",
        ResetType::Mixed => "--mixed",
        ResetType::Hard => "--hard",
    };
    let args = ["reset", reset_type, &remote_ref_str];

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

    let message = format!(
        "{:>9} {} : stash...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

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

    let message = format!(
        "{:>9} {} : pop stash...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

    let args = ["stash", "pop"];

    execute_cmd(&full_path, "git", &args)
}

fn execute_checkout_with_progress(
    input_path: &Path,
    toml_repo: &TomlRepo,
    force: bool,
    prefix: &str,
    progress_bar: &ProgressBar,
) -> anyhow::Result<()> {
    let rel_path = toml_repo.local.as_ref().unwrap();
    let full_path = input_path.join(rel_path);

    let message = format!(
        "{:>9} {} : checkout...",
        prefix,
        display_path(rel_path).bold().magenta()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

    // priority: commit/tag/branch(default-branch)
    let remote_ref = toml_repo.get_remote_ref(full_path.as_path())?;
    let remote_ref_str = match remote_ref.clone() {
        RemoteRef::Commit(commit) => commit,
        RemoteRef::Tag(tag) => tag,
        RemoteRef::Branch(branch) => branch,
    };
    let branch = match remote_ref {
        RemoteRef::Commit(commit) => format!("commits/{}", &commit[..7]),
        RemoteRef::Tag(tag) => format!("tags/{}", tag),
        RemoteRef::Branch(_) => toml_repo
            .branch
            .clone()
            .unwrap_or("invalid-branch".to_string()),
    };

    // don't need to checkout if current branch is the branch
    if let Ok(currnte_branch) = get_current_branch(full_path.as_path()) {
        if branch == currnte_branch {
            return Ok(());
        }
    }

    let message = format!(
        "{:>9} {} : checkout {}...",
        prefix,
        display_path(rel_path).bold().magenta(),
        &branch.blue()
    );
    progress_bar.set_message(fmt_msg_spinner(&message));

    // check if local branch already exists
    let output = execute_cmd(full_path.as_path(), "git", &["branch", "-l", &branch])?;
    let branch_exist = output.trim().contains(&branch);

    // create/checkout/reset branch
    let args = match (branch_exist, force) {
        (false, false) => vec!["checkout", "-B", &branch, &remote_ref_str, "--no-track"],
        (false, true) => vec![
            "checkout",
            "-B",
            &branch,
            &remote_ref_str,
            "--no-track",
            "-f",
        ],
        (true, false) => vec!["checkout", &branch],
        (true, true) => vec!["checkout", "-B", &branch, "-f"],
    };

    // exec cmd
    match execute_cmd(&full_path, "git", &args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Error: {}", e)),
    }
}
