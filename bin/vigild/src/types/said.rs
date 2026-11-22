use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct Said {
    failures: BTreeMap<&'static str, String>,
    health: BTreeMap<&'static str, String>,
    standing: BTreeMap<&'static str, String>,
    limits: BTreeMap<&'static str, String>,
    unconfirmed: BTreeSet<&'static str>,
}

impl Said {
    pub fn about(collectors: impl IntoIterator<Item = (&'static str, String)>) -> Self {
        let health: BTreeMap<&'static str, String> = collectors.into_iter().collect();
        let standing = health
            .iter()
            .filter(|(_, words)| *words != "ok")
            .map(|(name, words)| (*name, crate::helpers::health::in_words(words)))
            .collect();

        Said {
            failures: BTreeMap::new(),
            health,
            standing,
            limits: BTreeMap::new(),
            unconfirmed: BTreeSet::new(),
        }
    }

    pub fn failing(&mut self, collector: &'static str, error: &str) -> bool {
        self.failures
            .insert(collector, error.to_string())
            .as_deref()
            != Some(error)
    }

    pub fn failing_now(&self, collector: &'static str) -> bool {
        self.failures.contains_key(collector)
    }

    pub fn reading_again(&mut self, collector: &'static str) -> bool {
        self.failures.remove(collector).is_some()
    }

    pub fn health_moved(&mut self, collector: &'static str, words: String) -> Option<String> {
        let before = self.health.insert(collector, words.clone());

        match before {
            Some(before) if before == words => None,
            Some(before) => Some(before),
            None => Some(String::new()),
        }
    }

    pub fn opened(&mut self, collector: &'static str, why: String) {
        self.standing.insert(collector, why);
    }

    pub fn standing(&self, collector: &'static str) -> bool {
        self.standing.contains_key(collector)
    }

    pub fn closed(&mut self, collector: &'static str) -> Option<String> {
        self.standing.remove(collector)
    }

    pub fn limited(&mut self, collector: &'static str, words: String) -> bool {
        self.limits.insert(collector, words.clone()).as_deref() != Some(words.as_str())
    }

    pub fn limited_before(&mut self, collector: &'static str, words: String) {
        self.limits.insert(collector, words);
        self.unconfirmed.insert(collector);
    }

    pub fn limit_unconfirmed(&self, collector: &'static str) -> bool {
        self.unconfirmed.contains(collector)
    }

    pub fn limit_confirmed(&mut self, collector: &'static str) {
        self.unconfirmed.remove(collector);
    }

    pub fn unlimited(&mut self, collector: &'static str) -> Option<String> {
        self.limits.remove(collector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about_a_degraded_collector() -> Said {
        Said::about([
            ("network", "ok".to_string()),
            (
                "launches",
                "degraded:auditd has brought nothing".to_string(),
            ),
        ])
    }

    #[test]
    fn a_collector_that_was_unwell_when_the_daemon_started_is_one_it_has_spoken_about() {
        let mut said = about_a_degraded_collector();

        assert!(said.standing("launches"));
        assert_eq!(
            said.closed("launches").as_deref(),
            Some("degraded — auditd has brought nothing"),
            "the greeting raised a finding about it, something has to close that finding, and \
             what closes it quotes what it was"
        );
        assert!(
            !said.standing("network"),
            "a collector that was well has nothing standing to close"
        );
        assert_eq!(said.closed("network"), None);
    }

    #[test]
    fn what_a_complaint_stands_on_is_kept_in_the_words_that_raised_it() {
        let mut said = Said::default();

        said.opened(
            "network",
            "the reading failed — /proc/net/tcp: permission denied".into(),
        );

        assert_eq!(
            said.closed("network").as_deref(),
            Some("the reading failed — /proc/net/tcp: permission denied"),
            "a collector whose reading broke did not become degraded, and the closing half has \
             to say which of the two it closes"
        );
    }

    #[test]
    fn health_that_did_not_move_is_not_news_and_health_that_moved_names_what_it_was() {
        let mut said = about_a_degraded_collector();

        assert_eq!(said.health_moved("network", "ok".to_string()), None);
        assert_eq!(
            said.health_moved("launches", "ok".to_string()).as_deref(),
            Some("degraded:auditd has brought nothing"),
            "the closing half quotes what it was, so what it was has to survive the change"
        );
        assert_eq!(
            said.health_moved("launches", "ok".to_string()),
            None,
            "and the second reading of the same health says nothing twice"
        );
    }

    #[test]
    fn a_collector_nobody_named_at_startup_is_news_the_first_time_it_is_read() {
        let mut said = Said::default();

        assert!(said.health_moved("resources", "ok".to_string()).is_some());
        assert_eq!(said.health_moved("resources", "ok".to_string()), None);
    }

    #[test]
    fn the_same_failure_twice_is_logged_once_and_a_different_one_is_logged_again() {
        let mut said = Said::default();

        assert!(said.failing("network", "/proc/net/tcp: permission denied"));
        assert!(!said.failing("network", "/proc/net/tcp: permission denied"));
        assert!(said.failing("network", "/proc/net/tcp: no such file"));
        assert!(
            said.reading_again("network"),
            "a collector that failed and read again is worth one line"
        );
        assert!(
            !said.reading_again("network"),
            "and the readings after that are not"
        );
    }

    #[test]
    fn a_limit_that_did_not_move_is_not_news_and_one_that_moved_is() {
        let mut said = Said::default();

        assert!(said.limited("network", "the sockets of bob".into()));
        assert!(!said.limited("network", "the sockets of bob".into()));
        assert!(said.limited("network", "the sockets of bob and eve".into()));
        assert_eq!(
            said.unlimited("network").as_deref(),
            Some("the sockets of bob and eve")
        );
        assert_eq!(said.unlimited("network"), None);
    }

    #[test]
    fn a_limit_left_by_the_previous_run_is_looked_at_once_and_then_trusted() {
        let mut said = Said::default();

        said.limited_before("network", "the sockets of bob".into());
        assert!(said.limit_unconfirmed("network"));
        assert!(
            !said.limited("network", "the sockets of bob".into()),
            "the limit the previous run left is the one this run sees: nothing to say"
        );

        said.limit_confirmed("network");
        assert!(!said.limit_unconfirmed("network"));
    }
}
