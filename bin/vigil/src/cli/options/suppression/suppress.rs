use clap::Args;

use super::silencing::Silencing;
use crate::cli::style;
use crate::config;

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(styles = style::HELP)]
pub struct Suppress {
    #[command(subcommand)]
    pub doing: Silencing,
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
                file: asked.file.clone(),
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
