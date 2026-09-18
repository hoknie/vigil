use clap::Args;

use crate::cli::style;
use crate::config;

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Reading {
    #[arg(
        long,
        value_name = "PATH",
        default_value = config::DEFAULT_PATH,
        help = "The daemon's configuration, which says where suppressions are kept"
    )]
    pub config: String,
}
