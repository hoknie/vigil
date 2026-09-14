use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
}

impl ProtocolError {
    pub const MALFORMED_REQUEST: &'static str = "malformed_request";
    pub const UNKNOWN_QUERY: &'static str = "unknown_query";
    pub const UNKNOWN_COLLECTOR: &'static str = "unknown_collector";
    pub const REQUEST_TOO_LONG: &'static str = "request_too_long";
    pub const TOO_MANY_SESSIONS: &'static str = "too_many_sessions";
    pub const NOT_ALLOWED: &'static str = "not_allowed";
    pub const NOTHING_TO_ACT_ON: &'static str = "nothing_to_act_on";

    pub fn new(code: &str, message: impl Into<String>) -> Self {
        ProtocolError {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

impl std::error::Error for ProtocolError {}
