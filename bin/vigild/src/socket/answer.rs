use vigil_model::{Producer, ProtocolError, Request, Response, Rfc3339, SCHEMA_VERSION};

use super::State;

pub fn answer(request: &Request, state: &State, now: Rfc3339) -> Response {
    match request {
        Request::Status => Response::Status {
            schema_version: SCHEMA_VERSION,
            sent_at: now,
            producer: Producer {
                name: "vigil".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            host: Box::new(state.host().clone()),
            agent: Box::new(state.agent()),
        },

        Request::Snapshot { collector } => {
            if !state.knows_collector(collector) {
                return Response::Error {
                    error: ProtocolError::new(
                        ProtocolError::UNKNOWN_COLLECTOR,
                        format!(
                            "watching {}, not {collector:?}",
                            match state.collector_names().is_empty() {
                                true => "nothing".to_string(),
                                false => state.collector_names().join(", "),
                            }
                        ),
                    ),
                };
            }

            Response::Snapshot {
                collector: collector.clone(),
                snapshot: state.snapshot(collector).cloned(),
                refusal: state.refusal(collector),
            }
        }

        Request::Findings { limit } => Response::Findings {
            findings: state.latest_findings(*limit),
            dropped: state.findings_dropped(),
            capacity: state.findings_capacity(),
        },
    }
}

#[cfg(test)]
mod tests {
    use vigil_collect::Health;
    use vigil_model::CollectorState;

    use super::super::fixture;
    use super::*;
    use crate::socket::State;

    fn now() -> Rfc3339 {
        "2026-09-09T09:00:01.000Z".to_string()
    }

    #[test]
    fn the_status_answer_carries_who_is_speaking_and_what_it_is_watching_with() {
        let mut state = fixture::state();
        state.record_reading(fixture::reading(fixture::snapshot()));

        match answer(&Request::Status, &state, now()) {
            Response::Status {
                host,
                agent,
                producer,
                schema_version,
                sent_at,
            } => {
                assert_eq!(host.hostname, "app-01");
                assert_eq!(producer.name, "vigil");
                assert_eq!(schema_version, SCHEMA_VERSION);
                assert_eq!(sent_at, now());
                assert_eq!(agent.interval_seconds, 30);
                assert_eq!(agent.collectors.len(), 1);
                assert_eq!(agent.collectors[0].duration_ms, Some(5));
                assert_eq!(agent.collectors[0].items, 1);
            }
            other => panic!("answered with {other:?}"),
        }
    }

    #[test]
    fn a_collector_that_has_not_read_yet_answers_that_rather_than_an_empty_host() {
        let state = fixture::state();

        match answer(
            &Request::Snapshot {
                collector: "ports".into(),
            },
            &state,
            now(),
        ) {
            Response::Snapshot {
                snapshot, refusal, ..
            } => {
                assert!(
                    snapshot.is_none(),
                    "a snapshot the daemon has never taken must not arrive as an empty one"
                );
                assert!(
                    refusal.is_none(),
                    "and a collector whose turn has not come is not a collector that was refused"
                );
            }
            other => panic!("answered with {other:?}"),
        }
    }

    #[test]
    fn a_collector_that_could_not_read_says_why_in_the_answer_about_its_reading() {
        let mut state = fixture::state();
        state.record_health(
            "ports",
            &Health::Unavailable("/proc/net/tcp cannot be read".into()),
        );

        match answer(
            &Request::Snapshot {
                collector: "ports".into(),
            },
            &state,
            now(),
        ) {
            Response::Snapshot {
                snapshot, refusal, ..
            } => {
                assert!(snapshot.is_none());
                let refusal = refusal.expect("the answer carries the refusal, not the silence");
                assert_eq!(refusal.state, CollectorState::Unavailable);
                assert_eq!(refusal.reason, "/proc/net/tcp cannot be read");
            }
            other => panic!("answered with {other:?}"),
        }
    }

    #[test]
    fn a_reading_that_arrived_incomplete_carries_the_reason_beside_it() {
        let mut state = fixture::state();
        state.record_reading(fixture::reading(fixture::snapshot()));
        state.record_health("ports", &Health::Degraded("no owner for 2 socket(s)".into()));

        match answer(
            &Request::Snapshot {
                collector: "ports".into(),
            },
            &state,
            now(),
        ) {
            Response::Snapshot {
                snapshot, refusal, ..
            } => {
                assert!(snapshot.is_some(), "the reading itself still travels");
                assert_eq!(
                    refusal.expect("and says it is partial").reason,
                    "no owner for 2 socket(s)"
                );
            }
            other => panic!("answered with {other:?}"),
        }
    }

    #[test]
    fn every_way_a_collector_can_fall_into_trouble_leaves_words_to_show_the_reader() {
        let failed = |mut state: State| {
            state.record_failure("ports", now(), "/proc/net/tcp is not present on this system");
            state
        };
        let unwell = |mut state: State| {
            state.record_health("ports", &Health::Degraded("a partial reading".into()));
            state
        };
        let unavailable = |mut state: State| {
            state.record_health("ports", &Health::Unavailable("nothing to read".into()));
            state
        };

        for state in [
            failed(fixture::state()),
            unwell(fixture::state()),
            unavailable(fixture::state()),
            failed(unavailable(fixture::state())),
        ] {
            let refusal = state.refusal("ports").expect("trouble is never mute");
            assert!(refusal.state.is_trouble());
            assert!(!refusal.reason.is_empty(), "{:?}", refusal);
        }
    }

    #[test]
    fn a_collector_reading_normally_is_refused_nothing_and_says_nothing() {
        let mut state = fixture::state();
        state.record_health("ports", &Health::Ok);

        assert!(state.refusal("ports").is_none());
        assert!(
            state.refusal("files").is_none(),
            "and a name this agent does not watch is answered elsewhere, by its own refusal"
        );
    }

    #[test]
    fn a_collector_this_agent_does_not_have_is_refused_by_name() {
        let state = fixture::state();

        match answer(
            &Request::Snapshot {
                collector: "files".into(),
            },
            &state,
            now(),
        ) {
            Response::Error { error } => {
                assert_eq!(error.code, ProtocolError::UNKNOWN_COLLECTOR);
                assert!(error.message.contains("ports"), "{error}");
            }
            other => panic!("answered with {other:?}"),
        }
    }

    #[test]
    fn findings_come_back_newest_first_and_say_what_was_dropped() {
        let mut state = fixture::state();
        for title in ["first", "second", "third"] {
            state.record_findings(&[fixture::finding(title)]);
        }

        match answer(&Request::Findings { limit: Some(2) }, &state, now()) {
            Response::Findings {
                findings,
                dropped,
                capacity,
            } => {
                assert_eq!(findings.len(), 2);
                assert_eq!(findings[0].title, "third");
                assert_eq!(dropped, 0);
                assert!(capacity >= 3, "the cap travels with the answer");
            }
            other => panic!("answered with {other:?}"),
        }
    }
}
