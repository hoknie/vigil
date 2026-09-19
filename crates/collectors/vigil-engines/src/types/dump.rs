use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const PRESENT: &str = "present";

pub const ABSENT: &str = "absent";

pub const ANSWERED: &str = "answered";

pub const FAILED: &str = "failed";

pub const TIMED_OUT: &str = "timed_out";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dump {
    pub engine: String,
    pub state: String,
    pub taken_at: String,
    pub written_by: String,
    pub deadline_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(default)]
    pub asked: BTreeMap<String, Answer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    pub state: String,
    pub arguments: Vec<String>,
    pub milliseconds: u64,
    pub printed: String,
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

impl Dump {
    pub fn absent(engine: &str, taken_at: &str, deadline_seconds: u64, why: String) -> Dump {
        Dump {
            engine: engine.to_string(),
            state: ABSENT.to_string(),
            taken_at: taken_at.to_string(),
            written_by: super::engine::WRITER.to_string(),
            deadline_seconds,
            program: None,
            account: None,
            why: Some(why),
            asked: BTreeMap::new(),
        }
    }

    pub fn present(engine: &str, taken_at: &str, deadline_seconds: u64, program: &str) -> Dump {
        Dump {
            engine: engine.to_string(),
            state: PRESENT.to_string(),
            taken_at: taken_at.to_string(),
            written_by: super::engine::WRITER.to_string(),
            deadline_seconds,
            program: Some(program.to_string()),
            account: None,
            why: None,
            asked: BTreeMap::new(),
        }
    }

    pub fn on_this_host(&self) -> bool {
        self.state == PRESENT
    }

    pub fn answer(&self, subject: &str) -> Option<&Answer> {
        self.asked.get(subject)
    }

    pub fn unanswered(&self) -> Vec<String> {
        self.asked
            .iter()
            .filter(|(_, answer)| !answer.answered())
            .map(|(subject, answer)| format!("{subject} {}", answer.shortly()))
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_engine_that_is_not_installed_and_an_engine_that_did_not_answer_are_two_documents() {
        let missing = Dump::absent("podman", "2026-09-17T09:00:00.000Z", 10, "no podman".into());
        let broken = Dump::present("docker", "2026-09-17T09:00:00.000Z", 10, "/usr/bin/docker");

        assert!(!missing.on_this_host());
        assert!(broken.on_this_host());
        assert_ne!(
            missing.state, broken.state,
            "a host with no docker on it and a docker that refused to answer are different \
             facts, and a reader that cannot tell them apart reads silence as safety"
        );
    }

    #[test]
    fn a_command_that_did_not_finish_says_so_rather_than_reading_as_an_empty_list() {
        let answer = Answer {
            state: TIMED_OUT.to_string(),
            arguments: vec!["volume".into(), "ls".into()],
            milliseconds: 10_000,
            printed: String::new(),
            truncated: false,
            status: None,
            why: None,
        };

        assert!(!answer.answered());
        assert!(answer.shortly().contains("10000"), "{}", answer.shortly());
    }
}
