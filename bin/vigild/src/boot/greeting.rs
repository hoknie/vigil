use vigil_collect::Health;
use vigil_model::{Finding, Host};

use crate::Config;
use crate::helpers::agent_finding;
use crate::loops::Watch;
use crate::socket::switched_off_reason;

pub struct Greeting<'a> {
    pub config_path: &'a str,
    pub config: &'a Config,
    pub host: &'a Host,
    pub watches: &'a [Watch],
    pub switched_off: &'a [String],
    pub reporters: usize,
    pub damaged: usize,
}

impl Greeting<'_> {
    pub fn say(&self) {
        eprintln!(
            "vigild {} — host {}, install {}, state {}, {} reporter(s)",
            env!("CARGO_PKG_VERSION"),
            self.host.host_id,
            self.host.install_id,
            self.config.state_dir,
            self.reporters
        );
        self.say_collectors();
        self.say_health();
        if self.damaged > 0 {
            eprintln!(
                "  history: {} unreadable line(s) in the findings journal, skipped on open",
                self.damaged
            );
        }
    }

    pub fn findings(&self) -> Vec<Finding> {
        let mut findings = Vec::new();

        for watch in self.watches {
            match watch.health() {
                Health::Ok => {}
                Health::Degraded(detail) => findings.push(agent_finding::collector_degraded(
                    watch.name(),
                    &detail,
                    false,
                )),
                Health::Unavailable(detail) => findings.push(agent_finding::collector_degraded(
                    watch.name(),
                    &detail,
                    true,
                )),
            }
        }
        if self.damaged > 0 {
            findings.push(agent_finding::store_damaged(
                "findings",
                "the local findings history",
                self.damaged,
            ));
        }

        findings
    }

    fn say_collectors(&self) {
        match &self.config.collectors {
            None => eprintln!(
                "  collectors: all {} this build has. `collectors:` names none, which means all of them",
                self.watches.len()
            ),
            Some(_) if self.watches.is_empty() => eprintln!(
                "  collectors: NONE. `collectors:` in {} is empty: nothing is watched. History is kept and the console is answered",
                self.config_path
            ),
            Some(_) => eprintln!(
                "  collectors: {} ({} of {}, named in `collectors:`)",
                self.watches
                    .iter()
                    .map(|watch| watch.name())
                    .collect::<Vec<_>>()
                    .join(", "),
                self.watches.len(),
                self.watches.len() + self.switched_off.len()
            ),
        }
        for name in self.switched_off {
            eprintln!("  collector {name}: off — {}", switched_off_reason(name));
        }
    }

    fn say_health(&self) {
        for watch in self.watches {
            match watch.health() {
                Health::Ok => eprintln!("  collector {}: ok", watch.name()),
                Health::Degraded(detail) => {
                    eprintln!("  collector {}: degraded — {detail}", watch.name())
                }
                Health::Unavailable(detail) => {
                    eprintln!("  collector {}: unavailable — {detail}", watch.name())
                }
            }
        }
    }
}
