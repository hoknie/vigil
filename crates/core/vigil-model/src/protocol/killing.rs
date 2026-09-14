use serde::{Deserialize, Serialize};

use crate::Rfc3339;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Killing {
    Terminate,
    Kill,
    Destroy,
}

impl Killing {
    pub const ALL: &'static [Killing] = &[Killing::Terminate, Killing::Kill, Killing::Destroy];

    pub fn as_str(self) -> &'static str {
        match self {
            Killing::Terminate => "terminate",
            Killing::Kill => "kill",
            Killing::Destroy => "destroy",
        }
    }

    pub fn said(self) -> &'static str {
        match self {
            Killing::Terminate => "ask the process to stop (SIGTERM)",
            Killing::Kill => "stop the process now (SIGKILL)",
            Killing::Destroy => "close the socket, leave the process running",
        }
    }

    pub fn touches_the_process(self) -> bool {
        matches!(self, Killing::Terminate | Killing::Kill)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Killed {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    pub done: bool,
    pub said: String,
}

impl Killed {
    pub fn done(key: impl Into<String>, pid: u32, program: Option<String>, said: &str) -> Killed {
        Killed {
            key: key.into(),
            pid: Some(pid),
            program,
            done: true,
            said: said.to_string(),
        }
    }

    pub fn refused(key: impl Into<String>, said: impl Into<String>) -> Killed {
        Killed {
            key: key.into(),
            pid: None,
            program: None,
            done: false,
            said: said.into(),
        }
    }

    pub fn about(self, pid: Option<u32>, program: Option<String>) -> Killed {
        Killed {
            pid,
            program,
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillReport {
    pub killing: Killing,
    pub acted_at: Rfc3339,
    pub killed: Vec<Killed>,
}

impl KillReport {
    pub fn done(&self) -> usize {
        self.killed.iter().filter(|one| one.done).count()
    }

    pub fn refused(&self) -> usize {
        self.killed.len() - self.done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_way_of_killing_says_in_words_what_it_does_to_the_process() {
        for killing in Killing::ALL {
            assert!(
                !killing.said().is_empty(),
                "{} is offered to an operator with nothing written beside it, and a \
                 destructive choice with no sentence on it is a choice made by accident",
                killing.as_str()
            );
        }
        assert!(!Killing::Destroy.touches_the_process());
        assert!(Killing::Terminate.touches_the_process());
    }

    #[test]
    fn a_way_of_killing_round_trips_through_the_wire_name_it_is_asked_by() {
        for killing in Killing::ALL {
            let line = serde_json::to_string(killing).expect("serialises");
            assert_eq!(
                serde_json::from_str::<Killing>(&line).expect("reads back"),
                *killing
            );
        }
    }

    #[test]
    fn a_refusal_carries_the_key_it_refused_and_the_reason_rather_than_being_left_out() {
        let report = KillReport {
            killing: Killing::Destroy,
            acted_at: "2026-09-14T10:00:00.000Z".into(),
            killed: vec![
                Killed::done(
                    "tcp|0.0.0.0:4444",
                    30211,
                    Some("/tmp/.x/nc".into()),
                    "closed",
                ),
                Killed::refused(
                    "unix|/run/app.sock",
                    "a unix socket cannot be closed this way",
                ),
            ],
        };

        assert_eq!(report.done(), 1);
        assert_eq!(report.refused(), 1);
        assert_eq!(
            report.killed[1].key, "unix|/run/app.sock",
            "an operator who marked six rows and saw four die has to be told which two lived, \
             and why, by name"
        );
    }
}
