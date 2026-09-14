use clap::Subcommand;

use super::console::Console;
use super::suppression::Suppress;
use super::ui::Ui;

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
