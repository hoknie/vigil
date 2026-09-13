use clap::{Args, Parser, Subcommand};

use super::style;
use crate::ui::Screen;

pub const DEFAULT_SOCKET: &str = "/run/vigil/vigil.sock";

#[derive(Debug, Parser)]
#[command(
    name = "vigil",
    version,
    disable_version_flag = true,
    styles = style::HELP,
    about = "Read what the agent on this host has seen.",
    long_about = "\
Read what the agent on this host has seen: what is listening, who can log in, and what it
has found. Talks to the daemon over a local socket.",
    disable_help_subcommand = true,
    arg_required_else_help = true,
)]
pub struct Cli {
    #[arg(
        short = 'v',
        short_alias = 'V',
        long = "version",
        action = clap::ArgAction::Version,
        help = "Print the version and leave"
    )]
    pub version: (),

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Open the console against the daemon's local socket",
        long_about = "\
Open the console over the daemon's local socket. It opens on the main screen: one row per
section, with the number that opens it drawn on the row."
    )]
    Ui(Ui),

    #[command(
        about = "Print one screen as text and leave",
        long_about = "\
Print one screen as text and leave: for a script, a `watch` loop, a log, or a terminal that
cannot be drawn into."
    )]
    Capture(Console),

    #[command(hide = true)]
    Configure,

    #[command(hide = true)]
    Collector {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Console {
    #[arg(long, value_name = "PATH", default_value = DEFAULT_SOCKET)]
    pub socket: String,

    #[arg(
        long,
        value_name = "NAME",
        value_parser = screen,
        help = "Which screen to open it on",
        long_help = "\
Which screen to open it on: home, ports, accounts, programs, startup,
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

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Ui {
    #[command(flatten)]
    pub console: Console,

    #[arg(long, hide = true)]
    pub once: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opening {
    pub screen: Screen,
    pub difference: bool,
}

fn screen(name: &str) -> Result<Opening, String> {
    if name == DIFFERENCE {
        return Ok(Opening {
            screen: Screen::FINDINGS,
            difference: true,
        });
    }
    Screen::parse(name)
        .map(|screen| Opening {
            screen,
            difference: false,
        })
        .ok_or_else(|| SCREEN_NAMES.to_string())
}

const DIFFERENCE: &str = "difference";

const SCREEN_NAMES: &str = "\
\n  the sections are ports, accounts, programs, startup, firewall, summary and\
\n  findings,\
\n  and `home` is the screen that lists them\
\n  (`difference` too: the findings with the detail panel already open)";

pub const CONFIGURE_LIVES_IN_THE_DAEMON: &str = "\
vigil: `configure` lives in the daemon:

    vigild configure [PATH] [-f] [-d]

Writing a configuration asks every collector what it can watch here; the console
links no collectors. Both binaries ship in one package.";

pub const COLLECTOR_LIVES_IN_THE_DAEMON: &str = "\
vigil: switching a collector on or off lives in the daemon:

    vigild collector <name> enable [--config PATH] [-d]
    vigild collector <name> disable [--config PATH] [-d]

It writes /etc/vigil/vigil.yaml and starts what has to run on this host for that
reading, so it is the daemon's to do. Both binaries ship in one package.";

pub const UI_MOVED: &str = "\
vigil: opening the console is `vigil ui` now:

    vigil ui [--socket PATH] [--screen NAME]

The flags are unchanged.";

pub const CAPTURE_MOVED: &str = "\
vigil: printing one screen as text is `vigil capture` now:

    vigil capture [--socket PATH] [--screen NAME]

The same page and the same flags. Exit code: 0 the agent answered, 1 it did not.";

pub fn the_old_shape(arguments: &[String]) -> Option<&'static str> {
    let first = arguments.get(1)?;
    let word = first.split('=').next().unwrap_or(first);

    match word {
        "--once" => Some(CAPTURE_MOVED),
        "--socket" | "--screen" => match arguments
            .iter()
            .any(|word| word.split('=').next().unwrap_or(word) == "--once")
        {
            true => Some(CAPTURE_MOVED),
            false => Some(UI_MOVED),
        },
        _ => None,
    }
}
