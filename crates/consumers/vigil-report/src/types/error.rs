use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ReportError {
    Transient(String),
    Throttled(Duration),
    Refused { status: u16, reason: String },
    Malformed(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportError::Transient(what) => write!(f, "temporary failure: {what}"),
            ReportError::Throttled(wait) => write!(f, "asked to wait {wait:?}"),
            ReportError::Refused { status, reason } => write!(f, "refused ({status}): {reason}"),
            ReportError::Malformed(what) => write!(f, "malformed document: {what}"),
        }
    }
}

impl std::error::Error for ReportError {}
