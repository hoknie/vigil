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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::finding;

    fn temporary_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "vigil-outgoing-{}-{name}-{}.ndjson",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ))
    }

    fn small() -> Limits {
        Limits {
            findings: 10,
            journal_bytes: 64 * 1024,
        }
    }

    fn opened(path: &Path, limits: Limits) -> Outgoing {
        Outgoing::open("ndjson", path.to_path_buf(), limits).expect("opens")
    }

    fn findings(count: usize) -> Vec<Finding> {
        (0..count)
            .map(|index| finding(&format!("key-{index:04}"), "2026-09-09T10:00:00.000Z"))
            .collect()
    }

    fn keys(buffer: &Outgoing) -> Vec<String> {
        buffer
            .waiting()
            .into_iter()
            .map(|finding| finding.finding_key)
            .collect()
    }

    #[test]
    fn what_no_receiver_took_is_still_waiting_after_a_restart() {
        let path = temporary_path("restart");
        {
            let mut buffer = opened(&path, small());
            buffer.keep(&findings(3)).expect("keeps");
        }

        let buffer = opened(&path, small());

        assert_eq!(keys(&buffer), vec!["key-0000", "key-0001", "key-0002"]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_finding_the_receiver_took_does_not_come_back_a_second_time() {
        let path = temporary_path("taken");
        {
            let mut buffer = opened(&path, small());
            buffer.keep(&findings(3)).expect("keeps");
            buffer.delivered(3).expect("hands over");
        }

        let buffer = opened(&path, small());

        assert!(
            buffer.is_empty(),
            "a buffer that replays what was accepted turns one finding into a finding per restart"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_delivery_that_stopped_half_way_leaves_the_rest_in_the_order_it_had() {
        let path = temporary_path("half");
        let mut buffer = opened(&path, small());
        buffer.keep(&findings(4)).expect("keeps");

        buffer.delivered(2).expect("two of four were taken");

        assert_eq!(keys(&buffer), vec!["key-0002", "key-0003"]);
        let reopened = opened(&path, small());
        assert_eq!(keys(&reopened), vec!["key-0002", "key-0003"], "on disk too");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_ceiling_displaces_the_oldest_and_says_so_rather_than_refusing_the_newest() {
        let path = temporary_path("ceiling");
        let mut buffer = opened(&path, small());

        buffer.keep(&findings(10)).expect("keeps");
        let flow = buffer.keep(&findings(4)[..1]).expect("keeps the eleventh");

        assert!(
            matches!(flow, Flow::Dropping { dropped, .. } if dropped > 0),
            "a buffer that silently loses the oldest makes lost data look like no data: {flow:?}"
        );
        assert!(buffer.waiting().len() <= small().findings);
        assert_eq!(
            keys(&buffer).last().map(String::as_str),
            Some("key-0000"),
            "what was kept is the newest, and the newest here is the one just written"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_buffer_that_emptied_after_it_had_lost_findings_says_so_once() {
        let path = temporary_path("drained");
        let mut buffer = opened(&path, small());
        buffer.keep(&findings(12)).expect("keeps past the ceiling");

        let drained = buffer.delivered(usize::MAX).expect("all of it was taken");
        buffer.keep(&findings(1)).expect("keeps one more");
        let after = buffer.delivered(1).expect("taken");

        assert!(
            matches!(drained, Flow::Drained { dropped } if dropped > 0),
            "{drained:?}"
        );
        assert_eq!(
            after,
            Flow::Steady,
            "the finding about a dropping buffer closes once, not on every delivery after it"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_buffer_still_holding_findings_has_not_drained_however_many_went_out() {
        let path = temporary_path("partial-drain");
        let mut buffer = opened(&path, small());
        buffer.keep(&findings(12)).expect("keeps past the ceiling");

        let flow = buffer.delivered(1).expect("one was taken");

        assert_eq!(flow, Flow::Steady, "{flow:?}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_buffer_over_its_byte_ceiling_is_compacted_rather_than_grown() {
        let path = temporary_path("bytes");
        let by_bytes = Limits {
            findings: 10_000,
            journal_bytes: 2_048,
        };
        let mut buffer = opened(&path, by_bytes);

        let flow = buffer.keep(&findings(40)).expect("keeps");

        assert!(matches!(flow, Flow::Dropping { .. }), "{flow:?}");
        assert!(
            buffer.held().bytes.held <= by_bytes.journal_bytes,
            "twelve enormous findings fill a buffer as surely as ten thousand small ones"
        );
        assert!(!buffer.is_empty(), "and the newest ones are still there");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_torn_last_line_costs_that_finding_and_not_the_whole_buffer() {
        use std::io::Write;

        let path = temporary_path("torn");
        {
            let mut buffer = opened(&path, small());
            buffer.keep(&findings(2)).expect("keeps");
        }
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("reopens");
        file.write_all(b"{\"event_id\":\"half-writ").expect("tears");
        drop(file);

        let buffer = opened(&path, small());

        assert_eq!(keys(&buffer), vec!["key-0000", "key-0001"]);
        assert_eq!(
            buffer.damaged(),
            1,
            "the line nobody can read is counted and named, not thrown away in silence"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_file_left_over_a_ceiling_the_last_build_allowed_is_brought_under_it_on_open() {
        let path = temporary_path("shrunk");
        {
            let mut buffer = opened(
                &path,
                Limits {
                    findings: 1_000,
                    journal_bytes: 64 * 1024,
                },
            );
            buffer.keep(&findings(40)).expect("keeps");
        }

        let buffer = opened(&path, small());

        assert!(
            buffer.waiting().len() <= small().findings,
            "a ceiling that only applies to what this run wrote is not a ceiling"
        );
        assert_eq!(
            std::fs::read_to_string(&path)
                .expect("readable")
                .lines()
                .count(),
            buffer.waiting().len(),
            "and the file is what is held, not what was held"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn an_empty_buffer_names_no_oldest_finding_and_holds_no_bytes() {
        let path = temporary_path("empty");
        let buffer = opened(&path, small());

        let held = buffer.held();

        assert!(held.is_empty());
        assert_eq!(held.oldest_at, None);
        assert_eq!(held.bytes.held, 0);
        assert_eq!(buffer.name(), "ndjson");
        assert_eq!(
            buffer.path(),
            path,
            "and it names the file an operator reads"
        );
        let _ = std::fs::remove_file(&path);
    }
}
