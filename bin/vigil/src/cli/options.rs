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
has found. Talks to the daemon over a local socket. Nothing from the network is needed.",
    disable_help_subcommand = true,
    arg_required_else_help = true,
    after_help = "\
Examples:
  vigil ui                              open the console
  vigil ui --screen ports               open it on what is listening
  vigil capture --screen summary        print one page as text, for a script
  vigil ui --socket /tmp/vigil.sock     talk to a daemon socket somewhere else

The agent is `vigild`. Its configuration is written by `vigild configure`.",
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
section, with the number that opens it drawn on the row.

Four rungs: the main screen, a section, one of its lists, and the detail behind a row. The
arrows move whatever has them, and a caret marks it. Right or Enter goes one rung in, left or
Escape one rung out, and Escape from the top of a section comes back to the main screen. The
numbers work from any rung. Press ? inside for every key.",
        after_help = "\
Examples:
  vigil ui                              open it on the main screen
  vigil ui --screen accounts            open it on who can log in
  vigil ui --screen programs            open it on what has run here
  vigil ui --socket /tmp/vigil.sock     talk to a daemon socket somewhere else

To print a screen without opening the console: `vigil capture`."
    )]
    Ui(Ui),

    #[command(
        about = "Print one screen as text and leave",
        long_about = "\
Print one screen as text and leave: for a script, a `watch` loop, a log, or a terminal that
cannot be drawn into.

The same page the console draws, rendered by the same code and printed instead of shown. No
key is read, and the whole page is printed: a list is never cut off at the height of a
terminal.",
        after_help = "\
Examples:
  vigil capture                         the summary, as text
  vigil capture --screen findings       what the agent has found
  vigil capture --screen difference     the findings, with the detail panel open
  watch -n 30 vigil capture             a wall display

Exit code: 0 the agent answered, 1 it did not."
    )]
    Capture(Console),

    #[command(hide = true)]
    Configure,
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
summary, findings. `vigil ui` opens on home, `vigil capture` prints summary.

`difference` is accepted too and is not a screen: it opens the findings with
the detail panel already open. Kept for scripts."
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
            screen: Screen::Findings,
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
\n  the sections are ports, accounts, programs, startup, summary and findings,\
\n  and `home` is the screen that lists them\
\n  (`difference` too: the findings with the detail panel already open)";

pub const CONFIGURE_LIVES_IN_THE_DAEMON: &str = "\
vigil: `configure` lives in the daemon:

    vigild configure [PATH] [-f] [-d]

Writing a configuration asks every collector what it can watch here; the console
links no collectors. Both binaries ship in one package.";

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
