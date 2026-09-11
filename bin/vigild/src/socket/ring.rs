use std::collections::VecDeque;

use vigil_model::Finding;

pub struct Ring {
    items: VecDeque<Finding>,
    capacity: usize,
    dropped: u64,
    total: u64,
}

impl Ring {
    pub fn new(capacity: usize) -> Self {
        Ring {
            items: VecDeque::new(),
            capacity: capacity.max(1),
            dropped: 0,
            total: 0,
        }
    }

    pub fn push(&mut self, finding: Finding) {
        self.total += 1;
        if self.items.len() == self.capacity {
            self.items.pop_front();
            self.dropped += 1;
        }
        self.items.push_back(finding);
    }

    pub fn latest(&self, limit: Option<usize>) -> Vec<Finding> {
        let wanted = limit.unwrap_or(self.items.len()).min(self.items.len());
        self.items.iter().rev().take(wanted).cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn by_severity(&self) -> std::collections::BTreeMap<String, u64> {
        let mut counts = std::collections::BTreeMap::new();
        for finding in &self.items {
            *counts
                .entry(finding.severity.as_str().to_string())
                .or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::{Kind, Severity, State, Subject};

    use super::*;

    fn finding(title: &str) -> Finding {
        Finding {
            event_id: format!("id-{title}"),
            finding_key: format!("port.listen|{title}"),
            kind: Kind::from("port.listen.new".to_string()),
            severity: Severity::High,
            state: State::Open,
            observed_at: "2026-09-09T09:00:00.000Z".into(),
            first_seen_at: "2026-09-09T09:00:00.000Z".into(),
            occurrences: 1,
            title: title.to_string(),
            subject: Subject {
                object: "socket".into(),
                key: json!({}),
            },
            before: None,
            after: None,
            evidence: Vec::new(),
            redacted: Vec::new(),
            rule: None,
            labels: Default::default(),
        }
    }

    #[test]
    fn the_cap_discards_the_oldest_and_says_how_many() {
        let mut ring = Ring::new(2);

        ring.push(finding("first"));
        ring.push(finding("second"));
        ring.push(finding("third"));

        let latest = ring.latest(None);
        assert_eq!(latest.len(), 2);
        assert_eq!(latest[0].title, "third", "newest first");
        assert_eq!(latest[1].title, "second");
        assert_eq!(ring.dropped(), 1, "the loss has to be countable");
        assert_eq!(ring.total(), 3, "and so does what was raised");
    }

    #[test]
    fn asking_for_more_than_there_is_returns_what_there_is() {
        let mut ring = Ring::new(10);
        ring.push(finding("only"));

        assert_eq!(ring.latest(Some(50)).len(), 1);
        assert_eq!(ring.latest(Some(0)).len(), 0);
    }

    #[test]
    fn counts_severities_including_one_it_does_not_know() {
        let mut ring = Ring::new(10);
        let mut strange = finding("from-the-future");
        strange.severity = Severity::from("catastrophic".to_string());
        ring.push(finding("known"));
        ring.push(strange);

        let counts = ring.by_severity();

        assert_eq!(counts.get("high"), Some(&1));
        assert_eq!(
            counts.get("catastrophic"),
            Some(&1),
            "an unknown severity is counted, not dropped"
        );
    }
}
