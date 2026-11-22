use std::path::Path;

use vigil_collect::{Health, Standing};
use vigil_model::{Finding, Host};

use super::account::account_line;
use crate::Config;
use crate::helpers::agent_finding;
use crate::identity::Moved;
use crate::loops::Watch;
use crate::socket::switched_off_reason;

pub struct Greeting<'a> {
    pub config_path: &'a str,
    pub config: &'a Config,
    pub host: &'a Host,
    pub moved: Option<&'a Moved>,
    pub identity_said: &'a [String],
    pub watches: &'a [Watch],
    pub standings: &'a [(&'static str, Standing)],
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
        let account = crate::helpers::account::this();
        let name = crate::helpers::account::name_of(account.uid);
        if let Some(line) = account_line(&account, name.as_deref()) {
            eprintln!("{line}");
        }
        for line in self.identity_said {
            eprintln!("{line}");
        }
        self.say_reporters();
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
        if let Some(moved) = self.moved {
            findings.push(agent_finding::clone_suspected(moved));
        }

        findings.extend(health_findings(
            self.standings,
            &crate::helpers::account::named(),
        ));
        if self.damaged > 0 {
            findings.push(agent_finding::store_damaged(
                "findings",
                "the local findings history",
                self.damaged,
            ));
        }

        findings
    }

    fn say_reporters(&self) {
        for file in &self.config.apart.reporters {
            eprintln!(
                "  reporters: {} from {}",
                file.receivers.len(),
                file.path.display()
            );
        }
        if let Some(at) = &self.config.apart.reporters_at
            && !at.exists()
        {
            eprintln!(
                "  reporters: {} is not there, so no reporter is read from it",
                at.display()
            );
        }
    }

    fn say_collectors(&self) {
        let watched: Vec<&str> = self.watches.iter().map(|watch| watch.name()).collect();
        for line in collectors(self.config_path, self.config, &watched, self.switched_off) {
            eprintln!("{line}");
        }
    }

    fn say_health(&self) {
        for line in health_lines(self.standings) {
            eprintln!("{line}");
        }
    }
}

pub(super) fn health_lines(standings: &[(&'static str, Standing)]) -> Vec<String> {
    let mut said = Vec::new();
    for (name, standing) in standings {
        match (&standing.health, &standing.limit) {
            (Health::Ok, None) => said.push(format!("  collector {name}: ok")),
            (Health::Ok, Some(_)) => {}
            (Health::Degraded(detail), _) => {
                said.push(format!("  collector {name}: degraded — {detail}"))
            }
            (Health::Unavailable(detail), _) => {
                said.push(format!("  collector {name}: unavailable — {detail}"))
            }
        }
        if let Some(limit) = &standing.limit {
            said.push(format!("  collector {name}: limited — {limit}"));
        }
    }
    said
}

pub(super) fn health_findings(
    standings: &[(&'static str, Standing)],
    account: &str,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (name, standing) in standings {
        match &standing.health {
            Health::Ok => {}
            Health::Degraded(detail) => {
                findings.push(agent_finding::collector_degraded(name, detail, false))
            }
            Health::Unavailable(detail) => {
                findings.push(agent_finding::collector_degraded(name, detail, true))
            }
        }
        if let Some(limit) = &standing.limit {
            findings.push(agent_finding::collector_limited(
                name,
                &limit.said(),
                account,
            ));
        }
    }
    findings
}

pub(super) fn collectors(
    config_path: &str,
    config: &Config,
    watched: &[&str],
    switched_off: &[String],
) -> Vec<String> {
    let mut said = Vec::new();
    let at = config.apart.collectors_at.as_deref();
    if let Some(at) = at {
        said.push(format!("  collectors: read from {}", at.display()));
    }

    said.push(match (&config.collectors, at) {
        (None, _) => format!(
            "  collectors: all {} this build has. `collectors:` names none, which means all of them",
            watched.len()
        ),
        (Some(_), Some(at)) if watched.is_empty() => format!(
            "  collectors: NONE. {}: nothing is watched. History is kept and the console is answered",
            nothing_in(at, config)
        ),
        (Some(_), None) if watched.is_empty() => format!(
            "  collectors: NONE. `collectors:` in {config_path} is empty: nothing is watched. History is kept and the console is answered"
        ),
        (Some(_), _) => format!(
            "  collectors: {} ({} of {}, {})",
            watched.join(", "),
            watched.len(),
            watched.len() + switched_off.len(),
            match at {
                Some(_) => "each with a block that does not say `enabled: false`",
                None => "named in `collectors:`",
            }
        ),
    });

    for name in switched_off {
        said.push(format!(
            "  collector {name}: off — {}",
            switched_off_reason(config, name)
        ));
    }
    said
}

fn nothing_in(at: &Path, config: &Config) -> String {
    if !at.exists() {
        return format!("{} is not there", at.display());
    }
    match config.apart.collectors.is_empty() {
        true => format!("{} holds no block of a collector", at.display()),
        false => format!("every block in {} says `enabled: false`", at.display()),
    }
}
