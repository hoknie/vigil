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

    pub fn replace(&mut self, suppressions: Vec<Suppression>) {
        self.suppressions = suppressions;
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

    pub fn covers(&self, finding: &Finding, now: &str) -> bool {
        self.suppressions.iter().any(|entry| {
            entry.covers(
                &finding.finding_key,
                finding.kind.as_str(),
                finding.program(),
                now,
            )
        })
    }

    pub fn judge(&mut self, finding: &Finding, now: &str) -> Verdict {
        if self.covers(finding, now) {
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
    fn reading_a_silenced_finding_back_off_the_disk_does_not_raise_the_count_of_silences() {
        let mut policy = Policy::new(vec![suppression("port.listen|tcp|10.0.0.5:")]);
        let silenced = finding(
            "port.listen|tcp|10.0.0.5:9000",
            KnownKind::PortListenNew,
            Severity::Critical,
        );
        let now = "2026-09-09T10:00:00.000Z";

        assert!(policy.covers(&silenced, now));
        assert_eq!(
            policy.suppressed(),
            0,
            "asking whether the operator silenced this is not the same event as silencing it"
        );

        policy.judge(&silenced, now);
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

    fn held_by(key: &str, kind: KnownKind, before: Option<&str>, after: Option<&str>) -> Finding {
        let socket = |exe: &str| serde_json::json!({"protocol": "udp", "process": {"exe": exe}});
        Finding {
            before: before.map(socket),
            after: after.map(socket),
            ..finding(key, kind, Severity::Medium)
        }
    }

    fn tunnel() -> Suppression {
        Suppression {
            program: Some("netbird".into()),
            finding_key_match: Some("port.listen|udp|*".into()),
            reason: "vpn mesh changes its ports".into(),
            ..Suppression::default()
        }
    }

    #[test]
    fn a_tunnel_that_moves_its_ports_is_silenced_by_its_program_and_counted() {
        let mut policy = Policy::new(vec![tunnel()]);
        let now = "2026-10-30T10:00:00.000Z";

        let opened = held_by(
            "port.listen|udp|0.0.0.0:41234",
            KnownKind::PortListenNew,
            None,
            Some("/usr/bin/netbird"),
        );
        let closed = held_by(
            "port.listen|udp|0.0.0.0:39001",
            KnownKind::PortListenRemoved,
            Some("/usr/bin/netbird"),
            None,
        );
        let ssh = held_by(
            "port.listen|udp|0.0.0.0:41234",
            KnownKind::PortListenNew,
            None,
            Some("/usr/sbin/sshd"),
        );

        assert_eq!(policy.judge(&opened, now), Verdict::Suppressed);
        assert_eq!(policy.judge(&closed, now), Verdict::Suppressed);
        assert_eq!(policy.judge(&ssh, now), Verdict::Report);
        assert_eq!(policy.suppressed(), 2);
    }

    #[test]
    fn a_silenced_programs_socket_taken_over_by_another_program_is_reported() {
        let mut policy = Policy::new(vec![tunnel()]);

        let taken = held_by(
            "port.listen|udp|0.0.0.0:41234",
            KnownKind::PortListenOwnerChanged,
            Some("/usr/bin/netbird"),
            Some("/tmp/.x/nc"),
        );

        assert_eq!(
            policy.judge(&taken, "2026-10-30T10:00:00.000Z"),
            Verdict::Report,
            "the takeover is the finding the agent exists for, whoever held the port before"
        );
    }

    #[test]
    fn judging_findings_against_entries_with_programs_and_patterns_stays_in_microseconds() {
        let keyed: Vec<Suppression> = (0..50)
            .map(|at| suppression(&format!("port.listen|tcp|10.0.{at}.")))
            .collect();
        let mixed: Vec<Suppression> = (0..50)
            .map(|at| match at % 2 {
                0 => suppression(&format!("port.listen|tcp|10.0.{at}.")),
                _ => Suppression {
                    program: Some(format!("tunnel-{at}")),
                    finding_key_match: Some(format!("port.listen|*|*:{at}*")),
                    reason: "measured".into(),
                    ..Suppression::default()
                },
            })
            .collect();
        let findings: Vec<Finding> = (0..1000)
            .map(|at| {
                held_by(
                    &format!("port.listen|udp|0.0.0.0:{}", 30000 + at),
                    KnownKind::PortListenNew,
                    None,
                    Some("/usr/bin/netbird"),
                )
            })
            .collect();

        let per_finding = |entries: Vec<Suppression>| {
            let mut policy = Policy::new(entries);
            let started = std::time::Instant::now();
            for finding in &findings {
                policy.judge(finding, "2026-10-30T10:00:00.000Z");
            }
            started.elapsed() / findings.len() as u32
        };
        let before = per_finding(keyed);
        let after = per_finding(mixed);

        println!(
            "judge per finding, 50 entries: keys only {before:?}, half programs and patterns {after:?}"
        );
        assert!(
            after < std::time::Duration::from_micros(200),
            "a finding is judged on every round it is raised, and {after:?} is not cheap"
        );
    }
}
