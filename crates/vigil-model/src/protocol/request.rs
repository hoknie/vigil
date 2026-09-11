use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ProtocolError;

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
}

impl Request {
    pub const NAMES: &'static [&'static str] = &["status", "snapshot", "findings"];

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
    fn every_request_names_something_to_read_and_nothing_to_do() {
        assert_eq!(Request::NAMES, &["status", "snapshot", "findings"]);
        for name in Request::NAMES {
            assert!(
                Request::parse(&format!("{{\"query\":\"{name}\",\"collector\":\"ports\"}}"))
                    .is_ok(),
                "{name} is listed but does not parse"
            );
        }
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
