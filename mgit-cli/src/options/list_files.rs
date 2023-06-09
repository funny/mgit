use clap::Args;
use mgit::ops::ListFilesOptions as CoreListFilesOptions;
use mgit::option::CoreOptions;

use crate::cli::BaseOptions;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, Args)]
/// List tree
pub(crate) struct ListFilesOptions {
    #[clap(flatten)]
    base: BaseOptions,
}

impl From<ListFilesOptions> for CoreOptions {
    fn from(value: ListFilesOptions) -> Self {
        CoreOptions::new_list_files_options(value.base.path, value.base.config)
    }
}
