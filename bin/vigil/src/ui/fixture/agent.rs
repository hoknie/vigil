use vigil_model::{
    AgentBudget, AgentStatus, BufferStatus, CollectorState, CollectorStatus, FindingsSummary,
    ReporterStatus,
};

pub fn collector_off() -> CollectorStatus {
    CollectorStatus {
        name: "launches".into(),
        state: CollectorState::Off,
        reason: Some(
            "no line of the configuration on this host asks for this reading, so nothing is \
             looking at what it looks at: a decision somebody made, not a failure"
                .into(),
        ),
        last_run_at: None,
        duration_ms: None,
        items: 0,
        readings: 0,
        failures: 0,
        last_error: None,
        baseline: false,
        every_seconds: None,
        next_run_at: None,
        skipped: 0,
    }
}

pub fn collector(name: &str, every_seconds: u32, items: usize, skipped: u64) -> CollectorStatus {
    CollectorStatus {
        name: name.into(),
        state: CollectorState::Ok,
        reason: None,
        last_run_at: Some("2026-09-09T09:00:00.000Z".into()),
        duration_ms: Some(5),
        items,
        readings: 120,
        failures: 0,
        last_error: None,
        baseline: true,
        every_seconds: Some(every_seconds),
        next_run_at: Some("2026-09-09T09:00:11.000Z".into()),
        skipped,
    }
}

pub fn collector_degraded(name: &str, every_seconds: u32, reason: &str) -> CollectorStatus {
    CollectorStatus {
        name: name.into(),
        state: CollectorState::Degraded,
        reason: Some(reason.into()),
        last_run_at: None,
        duration_ms: None,
        items: 0,
        readings: 0,
        failures: 0,
        last_error: None,
        baseline: false,
        every_seconds: Some(every_seconds),
        next_run_at: None,
        skipped: 0,
    }
}

pub fn collector_failing(name: &str) -> CollectorStatus {
    CollectorStatus {
        last_error: Some("not permitted to read /proc/modules".into()),
        reason: Some("not permitted to read /proc/modules".into()),
        state: CollectorState::Degraded,
        failures: 1,
        ..collector(name, 300, 7, 0)
    }
}

pub fn collector_unavailable(name: &str) -> CollectorStatus {
    CollectorStatus {
        state: CollectorState::Unavailable,
        reason: Some(
            "auditd is not running on this host, so nothing is delivering launches".into(),
        ),
        last_run_at: None,
        duration_ms: None,
        next_run_at: None,
        readings: 0,
        baseline: false,
        items: 0,
        ..collector(name, 30, 0, 0)
    }
}

pub fn reporter(name: &str) -> ReporterStatus {
    ReporterStatus {
        name: name.into(),
        deliveries: 4,
        failures: 0,
        last_sent_at: Some("2026-09-09T09:00:00.000Z".into()),
        last_error: None,
    }
}

pub fn reporter_failing(name: &str) -> ReporterStatus {
    ReporterStatus {
        name: name.into(),
        deliveries: 0,
        failures: 2,
        last_sent_at: None,
        last_error: Some("the receiver answered 503".into()),
    }
}

pub fn agent() -> AgentStatus {
    AgentStatus {
        configuration_path: Some("/etc/vigil/vigil.yaml".to_string()),
        version: "0.1.0".into(),
        started_at: "2026-09-09T08:00:00.000Z".into(),
        interval_seconds: 30,
        budget: AgentBudget {
            duty_percent: Some(0.02),
            resident_kb: Some(12_288),
        },
        collectors: vec![
            collector("ports", 30, 2, 0),
            collector("users", 300, 11, 2),
            collector("processes", 30, 4, 0),
            collector("persistence", 300, 7, 0),
            collector("firewall", 60, 8, 0),
            collector("resources", 60, 5, 0),
            collector("containers", 60, 3, 0),
            collector("files", 300, 6, 0),
            collector_off(),
        ],
        reporters: vec![reporter("ndjson")],
        findings: FindingsSummary {
            retained: 3,
            capacity: 500,
            total: 3,
            dropped: 0,
            by_severity: [("critical".to_string(), 1u64), ("low".to_string(), 2)]
                .into_iter()
                .collect(),
        },
        silence: vigil_model::Silence {
            suppressed: 1,
            suppressions: vec!["port.listen|tcp|10.0.0.5:* — the staging api, expected".into()],
        },
        store: None,
        buffers: Some(vec![caught_up("ndjson")]),
        limitations: vec![
            "A thing this build cannot do yet is written here in a sentence about that long, so \
             the screen is drawn against a line of the size the daemon really sends."
                .into(),
        ],
    }
}

pub fn caught_up(receiver: &str) -> BufferStatus {
    BufferStatus {
        receiver: receiver.into(),
        pending: 0,
        pending_ceiling: 500,
        bytes: 0,
        bytes_ceiling: 4 * 1024 * 1024,
        dropped_total: 0,
        oldest_at: None,
    }
}

pub fn behind(receiver: &str) -> BufferStatus {
    BufferStatus {
        pending: 12,
        bytes: 8_664,
        oldest_at: Some("2026-09-09T08:59:30.000Z".into()),
        ..caught_up(receiver)
    }
}

pub fn losing(receiver: &str) -> BufferStatus {
    BufferStatus {
        pending: 500,
        bytes: 4 * 1024 * 1024,
        dropped_total: 7,
        ..behind(receiver)
    }
}
