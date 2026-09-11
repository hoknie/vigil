use vigil_model::Finding;

use crate::Suppression;

pub struct Policy {
    suppressions: Vec<Suppression>,
    suppressed: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Report,
    Suppressed,
}

impl Policy {
    pub fn new(suppressions: Vec<Suppression>) -> Self {
        Policy {
            suppressions,
            suppressed: 0,
        }
    }

    pub fn suppression_count(&self) -> usize {
        self.suppressions.len()
    }

    pub fn suppressed(&self) -> u64 {
        self.suppressed
    }

    pub fn describe_suppressions(&self) -> Vec<String> {
        self.suppressions
            .iter()
            .map(Suppression::describe)
            .collect()
    }

    pub fn judge(&mut self, finding: &Finding, now: &str) -> Verdict {
        let kind = finding.kind.as_str();

        if self
            .suppressions
            .iter()
            .any(|entry| entry.covers(&finding.finding_key, kind, now))
        {
            self.suppressed += 1;
            return Verdict::Suppressed;
        }

        Verdict::Report
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::{Kind, KnownKind, Severity, State, Subject};

    use super::*;

    fn finding(key: &str, kind: KnownKind, severity: Severity) -> Finding {
        Finding {
            event_id: "event".into(),
            finding_key: key.into(),
            kind: Kind::Known(kind),
            severity,
            state: State::Open,
            observed_at: "2026-09-09T10:00:00.000Z".into(),
            first_seen_at: "2026-09-09T10:00:00.000Z".into(),
            occurrences: 1,
            title: "something".into(),
            subject: Subject {
                object: "socket".into(),
                key: serde_json::json!({}),
            },
            before: None,
            after: None,
            evidence: Vec::new(),
            redacted: Vec::new(),
            rule: None,
            labels: Default::default(),
        }
    }

    fn suppression(prefix: &str) -> Suppression {
        Suppression {
            finding_key_prefix: Some(prefix.into()),
            reason: "expected here".into(),
            ..Suppression::default()
        }
    }

    #[test]
    fn what_the_operator_wrote_down_is_not_reported_and_is_counted() {
        let mut policy = Policy::new(vec![suppression("port.listen|tcp|10.0.0.5:")]);
        let now = "2026-09-09T10:00:00.000Z";

        assert_eq!(
            policy.judge(
                &finding(
                    "port.listen|tcp|10.0.0.5:9000",
                    KnownKind::PortListenNew,
                    Severity::Critical
                ),
                now
            ),
            Verdict::Suppressed,
            "an operator saying 'this is how the host is' outranks how loud the rule was"
        );
        assert_eq!(
            policy.judge(
                &finding(
                    "port.listen|tcp|0.0.0.0:4444",
                    KnownKind::PortListenNew,
                    Severity::Medium
                ),
                now
            ),
            Verdict::Report
        );
        assert_eq!(policy.suppressed(), 1);
    }

    #[test]
    fn with_nothing_written_down_everything_is_reported() {
        let mut policy = Policy::new(Vec::new());

        assert_eq!(
            policy.judge(
                &finding(
                    "port.listen|tcp|0.0.0.0:80",
                    KnownKind::PortListenNew,
                    Severity::Low
                ),
                "2026-09-09T10:00:00.000Z"
            ),
            Verdict::Report
        );
        assert_eq!(policy.suppressed(), 0);
    }
}
