use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use vigil_model::{Finding, Snapshot};

use super::baseline::Baselines;
use super::journal::Journal;
use super::limits::Limits;
use super::tally::Tally;
use crate::{Counted, Kept, Recorded, Store, StoreError};
pub struct FileStore {
    state: Mutex<State>,
}

type Identity = (String, String);

struct State {
    journal: Journal,
    baselines: Baselines,
    findings: BTreeMap<Identity, Finding>,
    by_age: BTreeSet<(String, Identity)>,
    limits: Limits,
    tally: Tally,
}

impl FileStore {
    pub fn open(state_dir: &Path) -> Result<Self, StoreError> {
        Self::with_limits(state_dir, Limits::default())
    }

    pub fn with_limits(state_dir: &Path, limits: Limits) -> Result<Self, StoreError> {
        let baselines = Baselines::open(state_dir.join("baselines"))?;
        let (journal, replay) = Journal::open(state_dir.join("findings").join("journal.ndjson"))?;
        let mut state = State {
            journal,
            baselines,
            findings: BTreeMap::new(),
            by_age: BTreeSet::new(),
            limits,
            tally: Tally {
                damaged_on_open: replay.damaged,
                journal_absent_on_open: replay.absent,
                ..Tally::default()
            },
        };
        for record in replay.records {
            state.remember(record);
        }

        Ok(FileStore {
            state: Mutex::new(state),
        })
    }
    pub fn damaged_on_open(&self) -> usize {
        self.state
            .lock()
            .map(|s| s.tally.damaged_on_open)
            .unwrap_or(0)
    }
    pub fn journal_absent_on_open(&self) -> bool {
        self.state
            .lock()
            .map(|s| s.tally.journal_absent_on_open)
            .unwrap_or(false)
    }
    pub fn compactions(&self) -> u64 {
        self.state.lock().map(|s| s.tally.compactions).unwrap_or(0)
    }
    pub fn baseline_directory(&self) -> Option<PathBuf> {
        self.state
            .lock()
            .ok()
            .map(|s| s.baselines.directory().to_path_buf())
    }
    pub fn journal_path(&self) -> Option<PathBuf> {
        self.state
            .lock()
            .ok()
            .map(|s| s.journal.path().to_path_buf())
    }
}

impl State {
    fn remember(&mut self, record: Finding) {
        let identity = identity(&record);
        self.forget(&identity);
        self.by_age
            .insert((record.observed_at.clone(), identity.clone()));
        self.findings.insert(identity, record);
    }

    fn forget(&mut self, identity: &Identity) -> bool {
        match self.findings.remove(identity) {
            Some(gone) => self.by_age.remove(&(gone.observed_at, identity.clone())),
            None => false,
        }
    }

    fn oldest(&self) -> Option<&str> {
        self.by_age.first().map(|(at, _)| at.as_str())
    }

    fn open_records(&self) -> u64 {
        self.findings
            .values()
            .filter(|finding| finding.state == vigil_model::State::Open)
            .count() as u64
    }

    fn enforce_limits(&mut self) -> Result<(), StoreError> {
        let over_count = self.findings.len() > self.limits.findings;
        let over_bytes = self.journal.bytes() > self.limits.journal_bytes;
        if !over_count && !over_bytes {
            return Ok(());
        }

        if over_count {
            let excess = self.findings.len().saturating_sub(self.limits.low_water());
            let doomed: Vec<Identity> = self
                .by_age
                .iter()
                .take(excess)
                .map(|(_, identity)| identity.clone())
                .collect();
            for identity in &doomed {
                if self.forget(identity) {
                    self.tally.dropped.at_the_ceiling += 1;
                }
            }
        }

        let live: Vec<Finding> = self.findings.values().cloned().collect();
        self.tally.compactions += 1;
        self.journal.rewrite(&live)
    }
}

impl Store for FileStore {
    fn baseline(&self, source: &str) -> Result<Option<Snapshot>, StoreError> {
        let state = self.state.lock().map_err(poisoned)?;
        state.baselines.read(source)
    }

    fn set_baseline(&self, snapshot: &Snapshot) -> Result<(), StoreError> {
        let state = self.state.lock().map_err(poisoned)?;
        state.baselines.write(snapshot)
    }

    fn forget_baseline(&self, source: &str) -> Result<bool, StoreError> {
        let state = self.state.lock().map_err(poisoned)?;
        state.baselines.forget(source)
    }

    fn record(&self, finding: &Finding) -> Result<Recorded, StoreError> {
        let mut state = self.state.lock().map_err(poisoned)?;

        let (outcome, stored) = match state.findings.get(&identity(finding)) {
            None => (Recorded::Created, finding.clone()),
            Some(previous) => {
                let occurrences = previous.occurrences + 1;
                let mut stored = finding.clone();
                stored.first_seen_at = previous.first_seen_at.clone();
                stored.occurrences = occurrences;
                (
                    Recorded::Repeated {
                        occurrences,
                        first_seen_at: stored.first_seen_at.clone(),
                    },
                    stored,
                )
            }
        };

        state.journal.append(&stored)?;
        state.remember(stored);
        state.enforce_limits()?;

        Ok(outcome)
    }

    fn open_findings(&self, limit: usize) -> Result<Vec<Finding>, StoreError> {
        let state = self.state.lock().map_err(poisoned)?;

        let mut open: Vec<Finding> = state
            .findings
            .values()
            .filter(|finding| finding.state == vigil_model::State::Open)
            .cloned()
            .collect();
        open.sort_by(|left, right| right.observed_at.cmp(&left.observed_at));
        open.truncate(limit);
        Ok(open)
    }

    fn resolve(&self, finding_key: &str, kind: &str, at: &str) -> Result<bool, StoreError> {
        let mut state = self.state.lock().map_err(poisoned)?;

        let identity = (finding_key.to_string(), kind.to_string());
        let Some(open) = state.findings.get(&identity) else {
            return Ok(false);
        };
        if open.state == vigil_model::State::Resolved {
            return Ok(false);
        }

        let mut closed = open.clone();
        closed.state = vigil_model::State::Resolved;
        closed.observed_at = at.to_string();

        state.journal.append(&closed)?;
        state.remember(closed);
        Ok(true)
    }

    fn prune(&self, older_than: &str) -> Result<u64, StoreError> {
        let mut state = self.state.lock().map_err(poisoned)?;

        let doomed: Vec<Identity> = state
            .by_age
            .iter()
            .take_while(|(observed_at, _)| observed_at.as_str() < older_than)
            .map(|(_, identity)| identity.clone())
            .collect();
        let dropped = doomed.len() as u64;
        for identity in &doomed {
            state.forget(identity);
        }

        if dropped > 0 {
            state.tally.dropped.past_the_window += dropped;
            let live: Vec<Finding> = state.findings.values().cloned().collect();
            state.journal.rewrite(&live)?;
        }
        Ok(dropped)
    }

    fn kept(&self) -> Result<Kept, StoreError> {
        let state = self.state.lock().map_err(poisoned)?;

        Ok(Kept {
            records: Counted::new(state.findings.len() as u64, state.limits.findings as u64),
            open: state.open_records(),
            bytes: Counted::new(state.journal.bytes(), state.limits.journal_bytes),
            oldest_at: state.oldest().map(str::to_string),
            dropped: state.tally.dropped,
            damaged: state.tally.damaged_on_open as u64,
            compactions: state.tally.compactions,
        })
    }
}

fn identity(finding: &Finding) -> Identity {
    (
        finding.finding_key.clone(),
        finding.kind.as_str().to_string(),
    )
}

fn poisoned<T>(_: std::sync::PoisonError<T>) -> StoreError {
    StoreError::Io("lock poisoned by a panic".into())
}
