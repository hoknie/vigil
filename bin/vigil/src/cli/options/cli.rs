use clap::Parser;

use super::command::Command;
use crate::cli::style;

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
