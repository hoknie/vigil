use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
    Unknown(String),
}

impl Severity {
    pub const KNOWN: &'static [Severity] = &[
        Severity::Info,
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ];

    pub fn as_str(&self) -> &str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
            Severity::Unknown(s) => s,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for Severity {
    fn from(s: String) -> Self {
        match s.as_str() {
            "info" => Severity::Info,
            "low" => Severity::Low,
            "medium" => Severity::Medium,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => Severity::Unknown(s),
        }
    }
}

impl From<Severity> for String {
    fn from(s: Severity) -> Self {
        s.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_levels_round_trip_and_unknown_ones_survive() {
        for level in Severity::KNOWN {
            let wire = level.as_str().to_string();
            assert_eq!(&Severity::from(wire), level);
        }
        assert_eq!(
            Severity::from("catastrophic".to_string()),
            Severity::Unknown("catastrophic".to_string())
        );
    }
}
