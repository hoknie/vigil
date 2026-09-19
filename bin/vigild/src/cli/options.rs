use clap::{Parser, Subcommand};

use super::style;
use crate::collector;
use crate::wizard::{DEFAULT_PATH, Options};

#[derive(Debug, Parser)]
#[command(
    name = "vigild",
    version,
    disable_version_flag = true,
    styles = style::HELP,
    about = "Watch this host, and report what changes about it.",
    long_about = "\
Watch this host: what is listening, who can log in, what runs at boot, what is running now.
Each reading is compared with the last one, and the difference becomes findings.",
    disable_help_subcommand = true,
    args_conflicts_with_subcommands = true,
    arg_required_else_help = true,
    after_help = "\
The console over the findings is `vigil ui`.",
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

    #[arg(
        value_name = "CONFIG",
        help = "The configuration file to watch this host with",
        long_help = "\
The configuration file to watch this host with.

It is named, never guessed. The shipped service names it on an installed host:
the systemd unit on Linux, the launchd job on macOS."
    )]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Ask this host what it can be watched with, and write the configuration for it",
        long_about = "\
Ask every collector what it can read on THIS host: the kernel it has, the files it may open,
the privileges it was started with, and say the answer here, one line per collector.

Then write the configuration: PATH, a file under collectors/ beside it for every collector that
can run here, and watch_fs.yaml, the files watched. A collector that cannot run here gets no
file, and the line above says why."
    )]
    Configure(Configure),

    #[command(about = "Switch one collector on or off in the configuration, and on this host")]
    Collector(Collector),

    #[command(hide = true)]
    Suppress {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
}

pub const SUPPRESS_LIVES_IN_THE_CONSOLE: &str = "\
vigild: `suppress` lives in the console:

    vigil suppress add \"<object>\" --reason \"...\"
    vigil suppress list
    vigil suppress remove \"<object>\"

The object key is read off the finding, and the finding is on the console's screen.
Both binaries ship in one package.";

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Collector {
    #[arg(value_name = "NAME", help = "Which collector")]
    pub name: String,

    #[command(subcommand)]
    pub doing: Switch,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Switch {
    #[command(about = "Watch it from now on")]
    Enable(Switching),

    #[command(about = "Stop watching it")]
    Disable(Switching),
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Switching {
    #[arg(
        long,
        value_name = "PATH",
        default_value = DEFAULT_PATH,
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

impl Collector {
    pub fn switching(&self) -> &Switching {
        match &self.doing {
            Switch::Enable(switching) | Switch::Disable(switching) => switching,
        }
    }

    pub fn options(&self) -> collector::Options {
        collector::Options {
            name: self.name.clone(),
            path: self.switching().config.clone(),
            dry_run: self.switching().dry_run,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Configure {
    #[arg(
        value_name = "PATH",
        default_value = DEFAULT_PATH,
        help = "The configuration file to write; the rest is written beside it"
    )]
    pub path: String,

    #[arg(
        short,
        long,
        help = "Replace the files that are already there",
        long_help = "\
Replace the files that are already there.

Without it an existing file is not touched, and the command says which. With it,
what was there is kept beside the new file as <file>.previous, and the file of a
collector that cannot run here is set aside the same way."
    )]
    pub force: bool,

    #[arg(
        short = 'd',
        long,
        help = "Print the files that would be written, to stdout, and write nothing",
        long_help = "\
Print the files that would be written, to stdout, and write nothing.

What each collector answered, and what a run would do with each file, is said
on stderr."
    )]
    pub dry_run: bool,
}

impl From<Configure> for Options {
    fn from(asked: Configure) -> Self {
        Options {
            path: asked.path,
            force: asked.force,
            dry_run: asked.dry_run,
        }
    }
}
