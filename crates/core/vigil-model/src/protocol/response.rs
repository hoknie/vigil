use serde::{Deserialize, Serialize};

use crate::{
    AgentStatus, ChangeReport, CollectorRefusal, ControlReport, Finding, Host, KillReport,
    Producer, ProtocolError, Rfc3339, SchemaVersion, Snapshot,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Response {
    Status {
        schema_version: SchemaVersion,
        sent_at: Rfc3339,
        producer: Producer,
        host: Box<Host>,
        agent: Box<AgentStatus>,
    },
    Snapshot {
        collector: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        snapshot: Option<Snapshot>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        refusal: Option<CollectorRefusal>,
    },
    Findings {
        findings: Vec<Finding>,
        dropped: u64,
        capacity: usize,
    },
    Killed {
        report: Box<KillReport>,
    },
    Changed {
        report: Box<ChangeReport>,
    },
    Controlled {
        report: Box<ControlReport>,
    },
    Error {
        error: ProtocolError,
    },
}

impl Response {
    pub fn to_line(&self) -> String {
        let mut line = serde_json::to_string(self).unwrap_or_else(|error| {
            let fallback = Response::Error {
                error: ProtocolError::new(
                    "unserialisable_answer",
                    format!("answer not serialised: {error}"),
                ),
            };
            serde_json::to_string(&fallback).unwrap_or_default()
        });
        line.push('\n');
        line
    }

    pub fn parse(line: &str) -> Result<Response, ProtocolError> {
        serde_json::from_str(line).map_err(|error| {
            ProtocolError::new(
                "malformed_answer",
                format!("answer is not one JSON document: {error}"),
            )
        })
    }
}
