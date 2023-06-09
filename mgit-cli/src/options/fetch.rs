use clap::{ArgAction, Args};
use mgit::ops::FetchOptions as CoreFetchOptions;
use mgit::options::CoreOptions;

use crate::cli::BaseOptions;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct FetchOptions {
    #[clap(flatten)]
    base: BaseOptions,

    /// Sets the number of threads to be used
    #[arg(short, long, default_value_t = 4, value_name = "NUMBER")]
    thread: usize,

    /// Do not report git status
    #[arg(long, action = ArgAction::SetTrue)]
    silent: bool,

    /// Deepen history of shallow clone
    #[arg(short, long, value_name = "NUMBER")]
    depth: Option<usize>,

    /// Ignore specified repositories for fetch
    #[arg(long)]
    ignore: Option<Vec<String>>,
}

impl From<FetchOptions> for CoreOptions {
    fn from(value: FetchOptions) -> Self {
        CoreOptions::new_fetch_options(
            value.base.path,
            value.base.config,
            Some(value.thread),
            Some(value.silent),
            value.depth,
            value.ignore,
        )
    }
}
