use std::path::{Path, PathBuf};

use vigil_config::{Suppression, files_in};

use super::Stamp;
use crate::config::{Config, load};

const KEPT: &str = "what is silenced stays what the files said when they last loaded";

type Seen = Vec<(PathBuf, Result<Stamp, String>)>;

#[derive(Debug, Default)]
pub struct Heard {
    pub said: Vec<String>,
    pub suppressions: Option<Vec<Suppression>>,
}

pub struct Silences {
    configuration: String,
    at: Option<PathBuf>,
    seen: Seen,
    applied: Vec<Suppression>,
    broken: bool,
}

impl Silences {
    #[cfg(test)]
    pub fn nothing() -> Silences {
        Silences {
            configuration: String::new(),
            at: None,
            seen: Vec::new(),
            applied: Vec::new(),
            broken: false,
        }
    }

    pub fn of(configuration: &str, config: &Config) -> Silences {
        let at = config.apart.suppressions_at.clone();
        Silences {
            configuration: configuration.to_string(),
            seen: seen(configuration, at.as_deref()),
            at,
            applied: config.every_suppression(),
            broken: false,
        }
    }

    pub fn look(&mut self) -> Heard {
        let mut heard = Heard::default();
        if self.configuration.is_empty() {
            return heard;
        }
        let now = seen(&self.configuration, self.at.as_deref());
        if now == self.seen {
            return heard;
        }
        self.seen = now;

        let config = match load(&self.configuration) {
            Ok(config) => config,
            Err(error) => {
                self.broken = true;
                heard.said.push(format!("{error}; {KEPT}"));
                return heard;
            }
        };
        if std::mem::take(&mut self.broken) {
            heard.said.push(format!(
                "suppressions: {} loads again, and what is silenced is what it names now",
                self.configuration
            ));
        }
        if config.apart.suppressions_at != self.at {
            self.at = config.apart.suppressions_at.clone();
            self.seen = seen(&self.configuration, self.at.as_deref());
        }

        let every = config.every_suppression();
        if every == self.applied {
            return heard;
        }
        heard.said.push(changed(&self.applied, &every));
        self.applied = every.clone();
        heard.suppressions = Some(every);
        heard
    }
}

fn seen(configuration: &str, at: Option<&Path>) -> Seen {
    let mut seen = vec![(PathBuf::from(configuration), Stamp::of(configuration))];
    let Some(at) = at else {
        return seen;
    };
    seen.push((at.to_path_buf(), Stamp::of(&at.display().to_string())));
    for file in files_in(at).unwrap_or_default() {
        if file.as_path() != at {
            let stamp = Stamp::of(&file.display().to_string());
            seen.push((file, stamp));
        }
    }
    seen
}

fn changed(before: &[Suppression], after: &[Suppression]) -> String {
    let added = after
        .iter()
        .filter(|one| !before.iter().any(|held| held == *one))
        .count();
    let taken_out = before
        .iter()
        .filter(|one| !after.iter().any(|held| held == *one))
        .count();
    format!(
        "suppressions: {} in force now ({added} new, {taken_out} taken out), taken up without a \
         restart",
        after.len()
    )
}
