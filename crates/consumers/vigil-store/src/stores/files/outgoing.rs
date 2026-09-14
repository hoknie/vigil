use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use vigil_model::Finding;

use super::journal::Journal;
use super::limits::Limits;
use crate::{Counted, Flow, Held, StoreError};

pub struct Outgoing {
    name: String,
    journal: Journal,
    waiting: VecDeque<Finding>,
    limits: Limits,
    dropped: u64,
    dropped_since_empty: u64,
    damaged: usize,
}

impl Outgoing {
    pub fn open(
        name: impl Into<String>,
        path: PathBuf,
        limits: Limits,
    ) -> Result<Self, StoreError> {
        let (journal, replay) = Journal::open(path)?;
        let mut outgoing = Outgoing {
            name: name.into(),
            journal,
            waiting: VecDeque::from(replay.records),
            limits,
            dropped: 0,
            dropped_since_empty: 0,
            damaged: replay.damaged,
        };
        outgoing.enforce()?;
        Ok(outgoing)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        self.journal.path()
    }

    pub fn damaged(&self) -> usize {
        self.damaged
    }

    pub fn waiting(&self) -> Vec<Finding> {
        self.waiting.iter().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }

    pub fn keep(&mut self, findings: &[Finding]) -> Result<Flow, StoreError> {
        if findings.is_empty() {
            return Ok(Flow::Steady);
        }

        self.journal.append_all(findings)?;
        self.waiting.extend(findings.iter().cloned());
        let dropped = self.enforce()?;

        match dropped {
            0 => Ok(Flow::Steady),
            _ => Ok(Flow::Dropping {
                dropped: self.dropped_since_empty,
                held: self.waiting.len() as u64,
            }),
        }
    }

    pub fn delivered(&mut self, count: usize) -> Result<Flow, StoreError> {
        let count = count.min(self.waiting.len());
        if count == 0 {
            return Ok(Flow::Steady);
        }

        for _ in 0..count {
            self.waiting.pop_front();
        }
        self.rewrite()?;

        if !self.waiting.is_empty() || self.dropped_since_empty == 0 {
            return Ok(Flow::Steady);
        }

        let dropped = self.dropped_since_empty;
        self.dropped_since_empty = 0;
        Ok(Flow::Drained { dropped })
    }

    pub fn held(&self) -> Held {
        Held {
            records: Counted::new(self.waiting.len() as u64, self.limits.findings as u64),
            bytes: Counted::new(self.journal.bytes(), self.limits.journal_bytes),
            dropped: self.dropped,
            damaged: self.damaged as u64,
            oldest_at: self
                .waiting
                .front()
                .map(|finding| finding.observed_at.clone()),
        }
    }

    fn enforce(&mut self) -> Result<u64, StoreError> {
        let over_records = self.waiting.len() > self.limits.findings;
        let over_bytes = self.journal.bytes() > self.limits.journal_bytes;
        if !over_records && !over_bytes {
            return Ok(0);
        }

        let mut dropped = 0;
        if over_records {
            dropped += self.forget_oldest(self.waiting.len() - self.limits.low_water());
        }
        if over_bytes {
            dropped += self.forget_oldest(self.over_by_bytes());
        }

        self.dropped += dropped;
        self.dropped_since_empty += dropped;
        self.rewrite()?;
        Ok(dropped)
    }

    fn over_by_bytes(&self) -> usize {
        let sizes: Vec<u64> = self.waiting.iter().map(line_bytes).collect();
        let mut total: u64 = sizes.iter().sum();
        let target = self.limits.low_water_bytes();

        let mut oldest = 0;
        while total > target && oldest < sizes.len() {
            total -= sizes[oldest];
            oldest += 1;
        }
        oldest
    }

    fn forget_oldest(&mut self, count: usize) -> u64 {
        let count = count.min(self.waiting.len());
        for _ in 0..count {
            self.waiting.pop_front();
        }
        count as u64
    }

    fn rewrite(&mut self) -> Result<(), StoreError> {
        let waiting: Vec<Finding> = self.waiting.iter().cloned().collect();
        self.journal.rewrite(&waiting)
    }
}

fn line_bytes(finding: &Finding) -> u64 {
    serde_json::to_string(finding)
        .map(|line| line.len() as u64 + 1)
        .unwrap_or(0)
}
