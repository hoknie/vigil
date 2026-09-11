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

    pub fn recall(&mut self, history: Vec<Finding>, held: u64) {
        let shown = history.len().min(self.capacity);
        self.total += held.max(shown as u64);
        self.dropped += held.saturating_sub(shown as u64);

        for finding in history.into_iter().take(shown).rev() {
            if self.items.len() == self.capacity {
                self.items.pop_front();
                self.dropped += 1;
            }
            self.items.push_back(finding);
        }
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

    fn aged(title: &str, observed_at: &str) -> Finding {
        let mut it = finding(title);
        it.observed_at = observed_at.to_string();
        it.first_seen_at = observed_at.to_string();
        it
    }

    #[test]
    fn what_the_journal_holds_and_what_this_run_raised_are_one_list_in_one_order() {
        let mut ring = Ring::new(10);

        ring.recall(
            vec![
                aged("yesterday-late", "2026-09-08T23:00:00.000Z"),
                aged("yesterday-early", "2026-09-08T08:00:00.000Z"),
            ],
            2,
        );
        ring.push(aged("just-now", "2026-09-09T09:00:00.000Z"));

        let latest = ring.latest(None);
        let titles: Vec<&str> = latest.iter().map(|it| it.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["just-now", "yesterday-late", "yesterday-early"],
            "newest first, whichever run raised it"
        );
    }

    #[test]
    fn what_the_journal_holds_beyond_the_ring_is_counted_rather_than_forgotten() {
        let mut ring = Ring::new(2);

        ring.recall(
            vec![
                aged("newest", "2026-09-08T12:00:00.000Z"),
                aged("middle", "2026-09-08T11:00:00.000Z"),
            ],
            9,
        );

        let titles: Vec<String> = ring
            .latest(None)
            .iter()
            .map(|it| it.title.clone())
            .collect();
        assert_eq!(titles, vec!["newest", "middle"]);
        assert_eq!(
            ring.dropped(),
            7,
            "nine in the journal and two on the screen is seven the reader cannot see"
        );
        assert_eq!(ring.total(), 9);
    }

    #[test]
    fn the_three_numbers_about_the_list_always_add_up() {
        let mut ring = Ring::new(3);
        ring.recall(
            vec![
                aged("a", "2026-09-08T12:00:00.000Z"),
                aged("b", "2026-09-08T11:00:00.000Z"),
                aged("c", "2026-09-08T10:00:00.000Z"),
            ],
            20,
        );

        for index in 0..4 {
            ring.push(aged(&format!("fresh-{index}"), "2026-09-09T09:00:00.000Z"));
        }

        assert_eq!(
            ring.len() as u64 + ring.dropped(),
            ring.total(),
            "held plus out of reach is everything there has been: {} + {} vs {}",
            ring.len(),
            ring.dropped(),
            ring.total()
        );
    }

    #[test]
    fn a_journal_with_nothing_open_in_it_leaves_nothing_behind_to_count() {
        let mut ring = Ring::new(500);

        ring.recall(Vec::new(), 0);

        assert_eq!(ring.len(), 0);
        assert_eq!(ring.dropped(), 0, "an empty journal hid nothing");
        assert_eq!(ring.total(), 0);
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
