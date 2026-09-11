use vigil_model::{
    AgentBudget, AgentStatus, CollectorState, CollectorStatus, FindingsSummary, ReporterStatus,
};

pub fn collector_off() -> CollectorStatus {
    CollectorStatus {
        name: "launches".into(),
        state: CollectorState::Off,
        reason: Some(
            "not named in `collectors:`, so nothing is watching what people run, from the \
             kernel's audit records: switched off, not failing"
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

pub fn agent() -> AgentStatus {
    AgentStatus {
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
            collector_off(),
        ],
        reporters: vec![ReporterStatus {
            name: "ndjson".into(),
            deliveries: 4,
            failures: 0,
            last_sent_at: Some("2026-09-09T09:00:00.000Z".into()),
            last_error: None,
        }],
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
            suppressions: vec![
                "port.listen|tcp|10.0.0.5:* — the staging api, expected".into(),
            ],
        },
        store: None,
        limitations: vec![
            "Nothing is marked resolved yet: a port that closed arrives as its own finding, beside the one that said it opened.".into(),
        ],
    }
}
