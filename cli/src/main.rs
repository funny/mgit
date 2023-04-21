mod commands;
mod config;
mod git;
mod utils;

use clap::CommandFactory;
use commands::{builtin_exec, Cli};

fn main() {
    let args = match Cli::command().try_get_matches() {
        Ok(r) => r,
        Err(e) => e.exit(),
    };

    let Some((cmd, subcommand_args)) =  args.subcommand() else {
        Cli::command().print_help().unwrap();
        return;
    };

    if let Some(exec) = builtin_exec(cmd) {
        exec(subcommand_args);
    }

    return;
}
