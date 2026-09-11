use std::collections::BTreeMap;

use vigil_collect::Health;

use crate::helpers::{health, rfc3339};

use super::Round;

impl Round {
    pub(super) fn take_health(
        &self,
        announce: bool,
        announced: &mut BTreeMap<&'static str, String>,
    ) {
        for watch in &self.watches {
            let state_of_it = watch.health();
            let name = watch.name();
            self.shared
                .with(|state| state.record_health(name, &state_of_it));
            if announce {
                self.announce(name, &state_of_it, announced);
            }
        }
    }

    fn announce(
        &self,
        name: &'static str,
        state_of_it: &Health,
        announced: &mut BTreeMap<&'static str, String>,
    ) {
        let now = health::describe(state_of_it);
        let before = announced.insert(name, now.clone());
        if before.as_deref() == Some(now.as_str()) {
            return;
        }

        match state_of_it {
            Health::Ok => eprintln!("{} collector {name}: ok again", rfc3339::now()),
            Health::Degraded(why) => {
                eprintln!("{} collector {name}: degraded — {why}", rfc3339::now())
            }
            Health::Unavailable(why) => {
                eprintln!("{} collector {name}: unavailable — {why}", rfc3339::now())
            }
        }
    }
}
