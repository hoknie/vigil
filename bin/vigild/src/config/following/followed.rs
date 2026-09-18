use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vigil_config::files_in;
use vigil_module::{Module, Settings};

use super::follower::Follower;
use super::{Looked, Stamp};
use crate::config::put::RESTART;
use crate::config::{Config, load};
use crate::helpers::rfc3339;

const KEPT: &str = "what is watched stays what the file said when it last loaded, and nothing is \
                    raised about it until the file loads again";

type Seen = Vec<(PathBuf, Result<Stamp, String>)>;

pub struct Followed {
    path: String,
    at: Option<PathBuf>,
    seen: Seen,
    read_at_start_up: BTreeMap<String, Value>,
    followers: Vec<Follower>,
    broken: bool,
}

impl Followed {
    #[cfg(test)]
    pub fn nothing() -> Followed {
        Followed {
            path: String::new(),
            at: None,
            seen: Vec::new(),
            read_at_start_up: BTreeMap::new(),
            followers: Vec::new(),
            broken: false,
        }
    }

    pub fn of(
        path: &str,
        seen: Result<Stamp, String>,
        config: &Config,
        modules: Vec<Box<dyn Module>>,
        watched: &[&str],
    ) -> Followed {
        let followers: Vec<Follower> = modules
            .into_iter()
            .filter(|module| module.follows_the_file() && watched.contains(&module.name()))
            .filter_map(|module| {
                let key = module.settings_key()?;
                Some(Follower {
                    applied: config.of_the_module(key),
                    key,
                    module,
                })
            })
            .collect();
        let named = named(&followers);
        let at = config.apart.collectors_at.clone();

        Followed {
            path: path.to_string(),
            seen: seen_with(path, seen, at.as_deref()),
            at,
            read_at_start_up: read_at_start_up(config, &named),
            followers,
            broken: false,
        }
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.followers
            .iter()
            .map(|follower| follower.module.name())
            .collect()
    }

    pub fn module(&self, name: &str) -> Option<&dyn Module> {
        self.followers
            .iter()
            .find(|follower| follower.module.name() == name)
            .map(|follower| follower.module.as_ref())
    }

    pub fn look(&mut self) -> Looked {
        let mut looked = Looked::default();
        if self.followers.is_empty() {
            return looked;
        }

        let now = seen_with(&self.path, Stamp::of(&self.path), self.at.as_deref());
        if now == self.seen {
            return looked;
        }
        self.seen = now;

        if let Some((_, Err(why))) = self.seen.first() {
            self.broken = true;
            looked.said.push(format!(
                "configuration {}: cannot be read now ({why}); {KEPT}",
                self.path
            ));
            return looked;
        }
        let config = match load(&self.path) {
            Ok(config) => config,
            Err(error) => {
                self.broken = true;
                looked.said.push(format!("{error}; {KEPT}"));
                return looked;
            }
        };

        if std::mem::take(&mut self.broken) {
            looked.said.push(format!(
                "configuration {}: loads again, and what is watched is what it names now",
                self.path
            ));
        }
        if config.apart.collectors_at != self.at {
            self.at = config.apart.collectors_at.clone();
            self.seen = seen_with(&self.path, Stamp::of(&self.path), self.at.as_deref());
        }
        self.say_what_waits_for_a_restart(&config, &mut looked);

        for follower in &mut self.followers {
            let block = config.of_the_module(follower.key);
            if block == follower.applied {
                continue;
            }
            follower.applied = block.clone();
            looked.refollowed.push((
                follower.module.name(),
                Settings::of(rfc3339::now, follower.key, block),
            ));
        }

        looked
    }

    fn say_what_waits_for_a_restart(&self, config: &Config, looked: &mut Looked) {
        let now = read_at_start_up(config, &named(&self.followers));

        let mut waiting: Vec<&str> = self
            .read_at_start_up
            .keys()
            .chain(now.keys())
            .filter(|key| self.read_at_start_up.get(*key) != now.get(*key))
            .map(String::as_str)
            .collect();
        waiting.sort_unstable();
        waiting.dedup();
        if waiting.is_empty() {
            return;
        }

        let place = match &self.at {
            Some(at) => format!("{}, {}", self.path, at.display()),
            None => self.path.clone(),
        };
        looked.said.push(format!(
            "configuration {place}: {} changed and is read at start-up only, so this daemon goes \
             on as it started until {RESTART}",
            waiting.join(", ")
        ));
    }
}

fn named(followers: &[Follower]) -> Vec<(&'static str, &'static str)> {
    followers
        .iter()
        .map(|follower| (follower.module.name(), follower.key))
        .collect()
}

fn seen_with(path: &str, stamp: Result<Stamp, String>, at: Option<&Path>) -> Seen {
    let mut seen = vec![(PathBuf::from(path), stamp)];
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

fn read_at_start_up(
    config: &Config,
    followed: &[(&'static str, &'static str)],
) -> BTreeMap<String, Value> {
    let mut read: BTreeMap<String, Value> = match serde_json::to_value(config) {
        Ok(Value::Object(fields)) => fields.into_iter().collect(),
        _ => BTreeMap::new(),
    };
    for taken_up in [vigil_config::SUPPRESSIONS_PATH, "suppressions"] {
        read.remove(taken_up);
    }

    if config.apart.collectors_at.is_none() {
        for (key, block) in &config.of_the_modules {
            if !followed.iter().any(|(_, followed)| followed == key) {
                read.insert(key.clone(), block.clone());
            }
        }
        return read;
    }

    for of_the_blocks in ["collectors", "schedule", "interval_seconds"] {
        read.remove(of_the_blocks);
    }
    for block in &config.apart.collectors {
        let settings = match followed.iter().any(|(name, _)| *name == block.name) {
            true => Value::Null,
            false => serde_json::to_value(&block.settings).unwrap_or(Value::Null),
        };
        read.insert(
            block.name.clone(),
            json!({
                "enabled": block.enabled,
                "schedule": block.schedule,
                "settings": settings,
            }),
        );
    }
    read
}
