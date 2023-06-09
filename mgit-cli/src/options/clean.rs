use clap::Args;
use mgit::ops::CleanOptions as CoreCleanOptions;
use mgit::options::CoreOptions;

use crate::cli::BaseOptions;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// Clean unused git repos
pub(crate) struct CleanOptions {
    #[clap(flatten)]
    base: BaseOptions,
}

impl From<CleanOptions> for CoreOptions {
    fn from(value: CleanOptions) -> Self {
        CoreOptions::new_clean_options(value.base.path, value.base.config)
    }
}
