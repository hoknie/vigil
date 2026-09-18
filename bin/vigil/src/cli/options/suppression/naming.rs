use clap::Args;

use crate::cli::style;
use crate::config;

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Naming {
    #[arg(
        value_name = "OBJECT",
        required = true,
        help = "The object key, as the console draws it on the finding"
    )]
    pub keys: Vec<String>,

    #[arg(
        long,
        value_name = "PATH",
        default_value = config::DEFAULT_PATH,
        help = "The daemon's configuration, which says where suppressions are kept"
    )]
    pub config: String,

    #[arg(
        short = 'd',
        long,
        help = "Print what would be done, and do none of it"
    )]
    pub dry_run: bool,
}
