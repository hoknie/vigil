use clap::{Args, Parser, Subcommand};

use super::style;
use crate::config;
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

    #[command(
        about = "Write down what this host is expected to do, so the agent stays quiet about it",
        long_about = "\
Write down what this host is expected to do, so the agent stays quiet about it.

An entry goes into `suppressions:` in the daemon's configuration. What it covers is judged the
moment a finding is raised: it reaches neither the local journal nor a receiver, and the
summary says how many were silenced. Every entry needs a reason, in your words. The daemon
reads the file at start, so a restart is named after every change."
    )]
    Suppress(Suppress),

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
pub struct Suppress {
    #[command(subcommand)]
    pub doing: Silencing,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Silencing {
    #[command(about = "Stop reporting the object a finding is about")]
    Add(Silence),

    #[command(about = "Report it again: take the entry out of the configuration")]
    Remove(Naming),

    #[command(about = "What the configuration silences now")]
    List(Reading),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Reading {
    #[arg(
        long,
        value_name = "PATH",
        default_value = config::DEFAULT_PATH,
        help = "The configuration file to read"
    )]
    pub config: String,
}

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
        help = "The configuration file to edit"
    )]
    pub config: String,

    #[arg(
        short = 'd',
        long,
        help = "Print what would be done, and do none of it"
    )]
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Silence {
    #[arg(
        value_name = "OBJECT",
        required = true,
        help = "The object key, as the console draws it on the finding"
    )]
    pub keys: Vec<String>,

    #[arg(
        short,
        long,
        required = true,
        value_name = "TEXT",
        help = "Why this host is expected to do it, in your words"
    )]
    pub reason: String,

    #[arg(
        short,
        long,
        help = "Cover every object whose key begins with what was named"
    )]
    pub prefix: bool,

    #[arg(
        short,
        long,
        value_name = "KIND",
        help = "Silence only this kind of finding about it"
    )]
    pub kind: Option<String>,

    #[arg(
        short,
        long,
        value_name = "WHEN",
        help = "Stop silencing it at this moment (2026-12-31 or 2026-12-31T00:00:00.000Z)"
    )]
    pub until: Option<String>,

    #[arg(
        long,
        value_name = "PATH",
        default_value = config::DEFAULT_PATH,
        help = "The configuration file to edit"
    )]
    pub config: String,

    #[arg(
        short = 'd',
        long,
        help = "Print what would be written, and write nothing"
    )]
    pub dry_run: bool,
}

impl Suppress {
    pub fn options(&self) -> config::Options {
        match &self.doing {
            Silencing::Add(asked) => config::Options {
                keys: asked.keys.clone(),
                reason: asked.reason.clone(),
                kind: asked.kind.clone(),
                until: asked.until.clone(),
                prefix: asked.prefix,
                path: asked.config.clone(),
                dry_run: asked.dry_run,
            },
            Silencing::Remove(asked) => config::Options {
                keys: asked.keys.clone(),
                path: asked.config.clone(),
                dry_run: asked.dry_run,
                ..config::Options::default()
            },
            Silencing::List(asked) => config::Options {
                path: asked.config.clone(),
                ..config::Options::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Console {
    #[arg(long, value_name = "PATH", default_value = DEFAULT_SOCKET)]
    pub socket: String,

    #[arg(
        long,
        value_name = "PATH",
        help = "The configuration to silence a finding in",
        long_help = "\
The configuration to silence a finding in, when it is not the one the daemon says it read.

The daemon answers with the file it was started with, and that is the file the findings screen
writes to. Name one here to edit another. The daemon reads its configuration at start, so a
restart is named after every change."
    )]
    pub config: Option<String>,

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
