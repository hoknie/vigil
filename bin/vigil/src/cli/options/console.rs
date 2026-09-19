use clap::Args;
use vigil_config::Installation;

use super::opening::Opening;
use super::screen::screen;
use crate::cli::style;
use crate::ui::Screen;

pub const DEFAULT_SOCKET: &str = Installation::here().socket;

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Console {
    #[arg(long, value_name = "PATH", default_value = DEFAULT_SOCKET)]
    pub socket: String,

    #[arg(
        long,
        value_name = "PATH",
        help = "The configuration to silence findings and watch paths in",
        long_help = "\
The configuration to silence findings and watch paths in, when it is not the one the daemon
says it read.

The daemon answers with the file it was started with, and that is the file the findings screen
and the watched files write to. Name one here to edit another. The daemon takes the watched
paths and the suppressions from it again on its next round; everything else is read at start."
    )]
    pub config: Option<String>,

    #[arg(
        long,
        value_name = "NAME",
        value_parser = screen,
        help = "Which screen to open it on",
        long_help = "\
Which screen to open it on: home, network, accounts, programs, startup,
firewall, summary, findings. `vigil ui` opens on home, `vigil capture` prints
summary."
    )]
    pub screen: Option<Opening>,
}

impl Console {
    pub fn opening(&self, unless_asked: Screen) -> Opening {
        self.screen.unwrap_or(Opening {
            screen: unless_asked,
            difference: false,
        })
    }
}
