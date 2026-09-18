use clap::Args;

use crate::cli::style;
use crate::config;

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
        short,
        long,
        value_name = "NAME",
        help = "The file under suppressions_path to write it to (default: console.yaml)"
    )]
    pub file: Option<String>,

    #[arg(
        long,
        value_name = "PATH",
        default_value = config::DEFAULT_PATH,
        help = "The daemon's configuration, which says where suppressions are kept"
    )]
    pub config: String,

    #[arg(
        short = 'd',
        long,
        help = "Print what would be written, and write nothing"
    )]
    pub dry_run: bool,
}
