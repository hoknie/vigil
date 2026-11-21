use clap::Args;

use crate::cli::style;

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Reading {
    #[arg(
        long,
        value_name = "PATH",
        default_value_t = super::super::places::default_configuration(),
        help = "The daemon's configuration, which says where suppressions are kept"
    )]
    pub config: String,
}
