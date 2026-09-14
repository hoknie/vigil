use clap::Parser;

use super::console::Console;
use crate::cli::style;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Ui {
    #[command(flatten)]
    pub console: Console,

    #[arg(long, hide = true)]
    pub once: bool,
}
