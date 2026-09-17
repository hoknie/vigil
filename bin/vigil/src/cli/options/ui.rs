use clap::Parser;

use super::console::Console;
use crate::cli::style;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Ui {
    #[command(flatten)]
    pub console: Console,

    #[arg(
        long,
        help = "Start with the mouse off",
        long_help = "\
Start with the mouse off, so the terminal keeps its own selection and copying.

The console reads clicks by default: a click moves the cursor, presses a button or picks an
option, and the wheel scrolls the list or the panel under the pointer. Every one of them is a
key as well, so nothing is lost with the mouse off. `m` turns it off and on while the console
is open, and a sheet opened for copying lets it go until it is closed."
    )]
    pub no_mouse: bool,

    #[arg(long, hide = true)]
    pub once: bool,
}
