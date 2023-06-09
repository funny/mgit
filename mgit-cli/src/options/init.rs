use clap::{ArgAction, Args};
use mgit::ops::InitOptions as CoreInitOptions;
use mgit::options::CoreOptions;

use crate::cli::BaseOptions;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
pub(crate) struct InitOptions {
    #[clap(flatten)]
    base: BaseOptions,

    /// Force remove git repos without prompt
    #[arg(long, action = ArgAction::SetTrue)]
    force: bool,
}

impl From<InitOptions> for CoreOptions {
    fn from(value: InitOptions) -> Self {
        CoreOptions::new_init_options(value.base.path, Some(value.force))
    }
}
