use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const DUMP_FILE: &str = "firewall.json";

pub const MACOS_DUMP_DIRECTORY: &str = "/usr/local/var/lib/vigil/firewall";

pub const MACOS_WRITER: &str = "/usr/local/libexec/vigil/vigil-firewall-dump";

pub const MACOS_JOB: &str = "vigil.firewall";

pub const ANSWERED: &str = "answered";

pub const FAILED: &str = "failed";

pub const TIMED_OUT: &str = "timed_out";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirewallDump {
    pub taken_at: String,
    pub written_by: String,
    pub deadline_seconds: u64,
    #[serde(default)]
    pub asked: BTreeMap<String, Answer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    pub state: String,
    pub program: String,
    pub arguments: Vec<String>,
    pub milliseconds: u64,
    pub printed: String,
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

impl FirewallDump {
    pub fn answer(&self, key: &str) -> Option<&Answer> {
        self.asked.get(key)
    }

    pub fn printed(&self, key: &str) -> Result<&str, String> {
        match self.asked.get(key) {
            None => Err(format!("{key} was not asked")),
            Some(answer) if answer.answered() => Ok(answer.printed.as_str()),
            Some(answer) => Err(format!("{key}: {}", answer.shortly())),
        }
    }
}

impl Answer {
    pub fn answered(&self) -> bool {
        self.state == ANSWERED
    }

    pub fn shortly(&self) -> String {
        match (self.state.as_str(), &self.why) {
            (TIMED_OUT, _) => format!("did not finish in {} ms", self.milliseconds),
            (_, Some(why)) => why.clone(),
            (state, None) => state.to_string(),
        }
    }
}
