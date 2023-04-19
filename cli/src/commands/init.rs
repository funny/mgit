use clap::ArgMatches;
use std::{env, path::PathBuf};

use super::{snapshot, SnapshotType};
use crate::utils::logger;

pub(crate) fn exec(args: &ArgMatches) {
    let input_path = match args.get_one::<String>("path") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().unwrap(),
    };

    logger::command_start("init", &input_path);

    if !input_path.is_dir() {
        logger::dir_not_found(&input_path);
        return;
    }

    let force = args.get_one::<bool>("force").unwrap_or(&false);

    let snapshot_type = SnapshotType::Branch;
    let config_file = input_path.join(".gitrepos");

    // check if .gitrepos exists
    if config_file.is_file() && !force {
        logger::dir_already_inited(&input_path);
        return;
    }

    snapshot::exec_snapshot(input_path, &config_file, snapshot_type, None);
}
