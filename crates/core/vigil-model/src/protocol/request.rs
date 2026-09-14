use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Killing, ProtocolError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "query", rename_all = "snake_case")]
pub enum Request {
    Status,
    Snapshot {
        collector: String,
    },
    Findings {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<usize>,
    },
    Kill {
        sockets: Vec<String>,
        killing: Killing,
    },
}

impl Request {
    pub const NAMES: &'static [&'static str] = &["status", "snapshot", "findings", "kill"];

    pub const READING: &'static [&'static str] = &["status", "snapshot", "findings"];

    pub fn acts_on_the_host(&self) -> bool {
        matches!(self, Request::Kill { .. })
    }

    pub fn parse(line: &str) -> Result<Request, ProtocolError> {
        let value: Value = serde_json::from_str(line).map_err(|error| {
            ProtocolError::new(
                ProtocolError::MALFORMED_REQUEST,
                format!("request is not one JSON document on one line: {error}"),
            )
        })?;

        let Some(object) = value.as_object() else {
            return Err(ProtocolError::new(
                ProtocolError::MALFORMED_REQUEST,
                "request is not a JSON object; expected {\"query\":\"status\"}",
            ));
        };
        match object.get("query").and_then(Value::as_str) {
            Some(name) if Request::NAMES.contains(&name) => {}
            Some(name) => {
                return Err(ProtocolError::new(
                    ProtocolError::UNKNOWN_QUERY,
                    format!(
                        "unknown query {name:?}; this build reads {}",
                        Request::NAMES.join(", ")
                    ),
                ));
            }
            None => {
                return Err(ProtocolError::new(
                    ProtocolError::UNKNOWN_QUERY,
                    format!(
                        "no \"query\" field; this build reads {}",
                        Request::NAMES.join(", ")
                    ),
                ));
            }
        }

        serde_json::from_value(value).map_err(|error| {
            ProtocolError::new(
                ProtocolError::MALFORMED_REQUEST,
                format!("query is missing a field it needs: {error}"),
            )
        })
    }
    pub fn to_line(&self) -> String {
        let mut line =
            serde_json::to_string(self).unwrap_or_else(|_| String::from("{\"query\":\"\"}"));
        line.push('\n');
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_one_request_does_something_on_the_host_and_it_is_named_here() {
        assert_eq!(Request::READING, &["status", "snapshot", "findings"]);
        for name in Request::READING {
            assert!(
                Request::parse(&format!("{{\"query\":\"{name}\",\"collector\":\"ports\"}}"))
                    .is_ok(),
                "{name} is listed but does not parse"
            );
            let request =
                Request::parse(&format!("{{\"query\":\"{name}\",\"collector\":\"ports\"}}"))
                    .expect("parses");
            assert!(
                !request.acts_on_the_host(),
                "{name} answers a question and must never grow a side effect"
            );
        }

        let acting: Vec<&&str> = Request::NAMES
            .iter()
            .filter(|name| !Request::READING.contains(name))
            .collect();

        assert_eq!(
            acting,
            vec![&"kill"],
            "this protocol held three questions and no verb until 2026-09-14, when the owner \
             put one in on purpose. A second verb arriving without that decision being taken \
             again is the failure this test exists to make loud."
        );
    }

    #[test]
    fn the_one_verb_names_what_it_kills_and_how_and_carries_nothing_a_daemon_would_run() {
        let request = Request::parse(
            "{\"query\":\"kill\",\"sockets\":[\"tcp|0.0.0.0:4444\"],\"killing\":\"terminate\"}",
        )
        .expect("parses");

        match &request {
            Request::Kill { sockets, killing } => {
                assert_eq!(sockets, &vec!["tcp|0.0.0.0:4444".to_string()]);
                assert_eq!(*killing, Killing::Terminate);
            }
            other => panic!("parsed as {other:?}"),
        }
        assert!(request.acts_on_the_host());
    }

    #[test]
    fn a_way_of_killing_this_build_has_not_heard_of_is_refused_rather_than_guessed_at() {
        let error = Request::parse(
            "{\"query\":\"kill\",\"sockets\":[\"tcp|0.0.0.0:80\"],\"killing\":\"reboot\"}",
        )
        .expect_err("must not be accepted");

        assert_eq!(error.code, ProtocolError::MALFORMED_REQUEST);
    }

    #[test]
    fn a_request_round_trips_through_its_line() {
        for request in [
            Request::Status,
            Request::Snapshot {
                collector: "ports".into(),
            },
            Request::Findings { limit: Some(20) },
            Request::Findings { limit: None },
            Request::Kill {
                sockets: vec!["tcp|0.0.0.0:4444".into()],
                killing: Killing::Destroy,
            },
        ] {
            let line = request.to_line();
            assert!(line.ends_with('\n'), "{line:?} is not one line");
            assert_eq!(Request::parse(line.trim_end()).expect("parses"), request);
        }
    }

    #[test]
    fn an_unknown_query_is_refused_by_name_instead_of_ignored() {
        let error = Request::parse("{\"query\":\"restart\"}").expect_err("must not be accepted");

        assert_eq!(error.code, ProtocolError::UNKNOWN_QUERY);
        assert!(error.message.contains("restart"), "{error}");
        assert!(error.message.contains("status"), "{error}");
    }

    #[test]
    fn text_that_is_not_a_request_is_answered_rather_than_guessed_at() {
        for line in ["", "not json", "[1,2,3]", "\"status\"", "{}"] {
            let error = Request::parse(line).expect_err("must not be accepted: {line:?}");
            assert!(
                error.code == ProtocolError::MALFORMED_REQUEST
                    || error.code == ProtocolError::UNKNOWN_QUERY,
                "{line:?} produced {error}"
            );
        }
    }

    #[test]
    fn a_query_missing_what_it_needs_says_which_field() {
        let error = Request::parse("{\"query\":\"snapshot\"}").expect_err("collector is needed");

        assert_eq!(error.code, ProtocolError::MALFORMED_REQUEST);
        assert!(error.message.contains("collector"), "{error}");
    }

    #[test]
    fn a_field_from_a_newer_console_does_not_make_the_request_unreadable() {
        let parsed = Request::parse("{\"query\":\"findings\",\"limit\":5,\"since\":\"tomorrow\"}")
            .expect("parses");

        assert_eq!(parsed, Request::Findings { limit: Some(5) });
    }
}
