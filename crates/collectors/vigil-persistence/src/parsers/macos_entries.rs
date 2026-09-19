use serde_json::json;
use vigil_collect::redact;
use vigil_model::Snapshot;

use super::crontab::CronEntry;
use super::entries::{SOURCE, WatchedScript, add_cron, add_scripts};
use super::launchd::LaunchdFacts;

pub const LAUNCHD: &str = "launchd";

const EVERY_PERSON: &str = "whoever logs in";

const WRITABLE_BY_ANY_ACCOUNT: &[&str] = &[
    "/tmp/",
    "/var/tmp/",
    "/private/tmp/",
    "/private/var/tmp/",
    "/private/var/folders/",
    "/Users/Shared/",
];

const WRITABLE_BY_ONE_PERSON: &str = "/Users/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Daemon,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    System,
    Vendor,
    Person,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchdJob {
    pub path: String,
    pub domain: Domain,
    pub scope: Scope,
    pub owner: Option<String>,
    pub readable: bool,
    pub facts: Result<LaunchdFacts, String>,
}

pub struct MacosPersistenceReading<'a> {
    pub jobs: &'a [LaunchdJob],
    pub cron: &'a [CronEntry],
    pub scripts: &'a [WatchedScript],
}

impl Domain {
    pub fn as_str(self) -> &'static str {
        match self {
            Domain::Daemon => "daemon",
            Domain::Agent => "agent",
        }
    }
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::System => "system",
            Scope::Vendor => "vendor",
            Scope::Person => "person",
        }
    }
}

impl LaunchdJob {
    pub fn run_as(&self) -> String {
        let named = self
            .facts
            .as_ref()
            .ok()
            .and_then(|facts| facts.user_name.clone());
        match (named, self.domain, &self.owner) {
            (Some(named), _, _) => named,
            (None, Domain::Daemon, _) => "root".to_string(),
            (None, Domain::Agent, Some(owner)) => owner.clone(),
            (None, Domain::Agent, None) => EVERY_PERSON.to_string(),
        }
    }

    pub fn label(&self) -> String {
        self.facts
            .as_ref()
            .ok()
            .and_then(|facts| facts.label.clone())
            .unwrap_or_else(|| {
                self.path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&self.path)
                    .trim_end_matches(".plist")
                    .to_string()
            })
    }

    pub fn starts_from_a_writable_path(&self) -> bool {
        let Ok(facts) = &self.facts else {
            return false;
        };
        let as_root = self.run_as() == "root";
        facts
            .program
            .iter()
            .chain(facts.inserted_libraries.iter())
            .any(|path| {
                WRITABLE_BY_ANY_ACCOUNT
                    .iter()
                    .any(|writable| path.starts_with(writable))
                    || (as_root && path.starts_with(WRITABLE_BY_ONE_PERSON))
            })
    }
}

pub fn macos_persistence_snapshot(
    taken_at: &str,
    reading: &MacosPersistenceReading<'_>,
) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    add_jobs(&mut snapshot, reading.jobs);
    add_cron(&mut snapshot, reading.cron);
    add_scripts(&mut snapshot, reading.scripts);

    snapshot
}

fn add_jobs(snapshot: &mut Snapshot, jobs: &[LaunchdJob]) {
    for job in jobs {
        let facts = job.facts.as_ref().ok();
        let (commands, commands_redacted) = match facts.filter(|facts| !facts.arguments.is_empty())
        {
            Some(facts) => {
                let clean = redact(&facts.arguments);
                (vec![clean.text], clean.redacted)
            }
            None => (Vec::new(), false),
        };

        snapshot.items.insert(
            format!("{LAUNCHD}|{}", job.path),
            json!({
                "name": job.label(),
                "path": job.path,
                "domain": job.domain.as_str(),
                "scope": job.scope.as_str(),
                "owner": job.owner,
                "readable": job.readable,
                "understood": job.facts.is_ok(),
                "refusal": job.facts.as_ref().err(),
                "program": facts.and_then(|facts| facts.program.clone()),
                "commands": commands,
                "commands_redacted": commands_redacted,
                "run_as": job.run_as(),
                "schedule": facts.map(LaunchdFacts::schedule),
                "disabled": facts.is_some_and(|facts| facts.disabled),
                "inserted_libraries": facts
                    .map(|facts| facts.inserted_libraries.clone())
                    .unwrap_or_default(),
                "writable_path": job.starts_from_a_writable_path(),
            }),
        );
    }
}
