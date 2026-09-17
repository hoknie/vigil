use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AccountChange, Controlling, KillTarget, Killing, ProtocolError};

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
        #[serde(default)]
        sockets: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        programs: Vec<String>,
        killing: Killing,
    },
    Change {
        #[serde(default)]
        changes: Vec<AccountChange>,
    },
    Control {
        #[serde(default)]
        keys: Vec<String>,
        controlling: Controlling,
    },
}

impl Request {
    pub const NAMES: &'static [&'static str] = &[
        "status", "snapshot", "findings", "kill", "change", "control",
    ];

    pub const READING: &'static [&'static str] = &["status", "snapshot", "findings"];

    pub fn kill(target: KillTarget, keys: Vec<String>, killing: Killing) -> Request {
        match target {
            KillTarget::Socket => Request::Kill {
                sockets: keys,
                programs: Vec::new(),
                killing,
            },
            KillTarget::Program => Request::Kill {
                sockets: Vec::new(),
                programs: keys,
                killing,
            },
        }
    }

    pub fn control(keys: Vec<String>, controlling: Controlling) -> Request {
        Request::Control { keys, controlling }
    }

    pub fn acts_on_the_host(&self) -> bool {
        matches!(
            self,
            Request::Kill { .. } | Request::Change { .. } | Request::Control { .. }
        )
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
