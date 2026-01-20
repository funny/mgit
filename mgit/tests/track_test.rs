use mgit::error::MgitResult;
use mgit::ops;
use mgit::ops::{SyncOptions, TrackOptions};
use std::path::PathBuf;

use crate::common::{
    create_test_dir, exec_cmd, failed_message, TestProgress, TomlBuilder, DEFAULT_BRANCH,
    SBERT_REPO,
};

mod common;

/// 测试内容：
///     1、运行命令:
///         - mgit track <path>
///     5、根目录是仓库
///
/// 测试目录结构:
///   test_track_simple(.git)
///     ├─.gitrepos
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_track_simple() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_track_simple");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-1",
            &SBERT_REPO,
            Some("attention_highlight"),
            None,
            None,
        )
        .join_repo("foobar-2", &SBERT_REPO, Some("character_bert"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize repositories, with no-track
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    let cur_branch_args = ["branch", "--show-current"];
    let tracking_args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "@{u}"];
    let root_path = &path;
    let foobar_1_path = &path.join("foobar-1");
    let foobar_2_path = &path.join("foobar-2");
    let invald_name = "invalid".to_string();

    // root: master untracked
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // foobar-1: master untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: master untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    let toml_string = toml_string.replace(
        r#"commit = "dc1d3dbb0383f72fd4b7adcd1a4d54abf557175d""#,
        r#"branch = "master""#,
    );
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // track command
    ops::track(
        TrackOptions::new(Some(input_path), None::<PathBuf>, None),
        TestProgress,
    )
    .await?;

    // root: foobar untracked,  checkout failed
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    let tracking_branch = exec_cmd(root_path, "git", &tracking_args).unwrap_or(invald_name.clone());
    assert_eq!(tracking_branch.trim(), "origin/master");

    // foobar-1: commits/90296ef untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    let tracking_branch =
        exec_cmd(foobar_1_path, "git", &tracking_args).unwrap_or(invald_name.clone());
    assert_eq!(tracking_branch.trim(), "origin/attention_highlight");

    // foobar-2: tags/1.0.3 untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    let tracking_branch =
        exec_cmd(foobar_2_path, "git", &tracking_args).unwrap_or(invald_name.clone());
    assert_eq!(tracking_branch.trim(), "origin/character_bert");

    Ok(())
}

/// 测试内容：
///     1、运行命令:
///         - mgit track <path> --igonre <path> --igonre <path>
///     5、根目录是仓库
///
/// 测试目录结构:
///   test_track_ignore(.git)
///     ├─.gitrepos
///     ├─foobar-1 (.git)
///     └─foobar-2 (.git)
#[tokio::test]
async fn cli_track_ignore() -> MgitResult<()> {
    let tmp_dir = create_test_dir("test_track_ignore");
    let path = tmp_dir.path().to_path_buf();
    let input_path = path.to_str().unwrap();

    std::fs::create_dir_all(&path).unwrap();

    let toml_string = TomlBuilder::default()
        .default_branch("develop")
        .join_repo(".", &SBERT_REPO, Some("master"), None, None)
        .join_repo(
            "foobar-1",
            &SBERT_REPO,
            Some("attention_highlight"),
            None,
            None,
        )
        .join_repo("foobar-2", &SBERT_REPO, Some("character_bert"), None, None)
        .build();
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // initialize repositories, with no-track
    ops::sync_repo(
        SyncOptions::new(
            Some(input_path),
            None::<PathBuf>,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
            Some(true),
        ),
        TestProgress,
    )
    .await?;

    let cur_branch_args = ["branch", "--show-current"];
    let tracking_args = ["rev-parse", "--symbolic-full-name", "--abbrev-ref", "@{u}"];
    let root_path = &path;
    let foobar_1_path = &path.join("foobar-1");
    let foobar_2_path = &path.join("foobar-2");
    let invald_name = "invalid".to_string();

    // root: master untracked
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // foobar-1: master untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: master untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_2_path, "git", &tracking_args).is_err());

    let toml_string = toml_string.replace(
        r#"commit = "dc1d3dbb0383f72fd4b7adcd1a4d54abf557175d""#,
        r#"branch = "master""#,
    );
    let config_file = path.join(".gitrepos");
    std::fs::write(config_file, toml_string.trim()).expect(failed_message::WRITE_FILE);

    // track command
    ops::track(
        TrackOptions::new(
            Some(input_path),
            None::<PathBuf>,
            Some([".", "foobar-1"].map(|s| s.to_string()).to_vec()),
        ),
        TestProgress,
    )
    .await?;

    // root: foobar untracked,  checkout failed
    let branch = exec_cmd(root_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(root_path, "git", &tracking_args).is_err());

    // foobar-1: master untracked
    let branch = exec_cmd(foobar_1_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    assert!(exec_cmd(foobar_1_path, "git", &tracking_args).is_err());

    // foobar-2: tags/1.0.3 untracked
    let branch = exec_cmd(foobar_2_path, "git", &cur_branch_args).unwrap_or(invald_name.clone());
    assert_eq!(branch.trim(), DEFAULT_BRANCH);
    let tracking_branch =
        exec_cmd(foobar_2_path, "git", &tracking_args).unwrap_or(invald_name.clone());
    assert_eq!(tracking_branch.trim(), "origin/character_bert");

    Ok(())
}
