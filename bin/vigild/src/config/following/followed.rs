use std::collections::BTreeMap;

use serde_json::Value;
use vigil_module::{Module, Settings};

use super::follower::Follower;
use super::{Looked, Stamp};
use crate::config::put::RESTART;
use crate::config::{Config, load};
use crate::helpers::rfc3339;

const KEPT: &str = "what is watched stays what the file said when it last loaded, and nothing is \
                    raised about it until the file loads again";

pub struct Followed {
    path: String,
    seen: Result<Stamp, String>,
    read_at_start_up: BTreeMap<String, Value>,
    followers: Vec<Follower>,
    broken: bool,
}

impl Followed {
    #[cfg(test)]
    pub fn nothing() -> Followed {
        Followed {
            path: String::new(),
            seen: Err(String::new()),
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
        let keys: Vec<&str> = followers.iter().map(|follower| follower.key).collect();

        Followed {
            path: path.to_string(),
            seen,
            read_at_start_up: read_at_start_up(config, &keys),
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

        let stamp = Stamp::of(&self.path);
        if stamp == self.seen {
            return looked;
        }
        self.seen = stamp.clone();

        if let Err(why) = stamp {
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
        let keys: Vec<&str> = self.followers.iter().map(|follower| follower.key).collect();
        let now = read_at_start_up(config, &keys);

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

        looked.said.push(format!(
            "configuration {}: {} changed and is read at start-up only, so this daemon goes on \
             as it started until {RESTART}",
            self.path,
            waiting.join(", ")
        ));
    }
}

fn read_at_start_up(config: &Config, followed: &[&str]) -> BTreeMap<String, Value> {
    let mut read: BTreeMap<String, Value> = match serde_json::to_value(config) {
        Ok(Value::Object(fields)) => fields.into_iter().collect(),
        _ => BTreeMap::new(),
    };
    for taken_up in [vigil_config::SUPPRESSIONS_PATH, "suppressions"] {
        read.remove(taken_up);
    }
    for (key, block) in &config.of_the_modules {
        if !followed.contains(&key.as_str()) {
            read.insert(key.clone(), block.clone());
        }
    }

    read
}
