use std::fmt;

const ABSENT_UNDER_SYSTEMD: &[&str] = &[
    "Is the agent running?  systemctl status vigild",
    "Is this the socket it opened?  the socket_path line of its configuration",
    "Point the console at another one:  vigil ui --socket /path/to/vigil.sock",
];

const ABSENT_UNDER_LAUNCHD: &[&str] = &[
    "Is the agent running?  launchctl print system/vigil.vigild",
    "Is this the socket it opened?  the socket_path line of its configuration",
    "Point the console at another one:  vigil ui --socket /path/to/vigil.sock",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trouble {
    pub path: String,
    pub kind: TroubleKind,
    pub cause: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TroubleKind {
    Absent,
    Forbidden,
    Unreadable,
}

impl Trouble {
    pub fn new(path: &str, kind: TroubleKind, cause: impl Into<String>) -> Self {
        Trouble {
            path: path.to_string(),
            kind,
            cause: cause.into(),
        }
    }

    pub fn headline(&self) -> String {
        match self.kind {
            TroubleKind::Absent => {
                format!("The agent is not answering at {}.", self.path)
            }
            TroubleKind::Forbidden => {
                format!("This user may not open {}.", self.path)
            }
            TroubleKind::Unreadable => {
                format!("The agent at {} answered something unreadable.", self.path)
            }
        }
    }

    pub fn what_to_try(&self) -> &'static [&'static str] {
        match self.kind {
            TroubleKind::Absent => match cfg!(target_os = "macos") {
                true => ABSENT_UNDER_LAUNCHD,
                false => ABSENT_UNDER_SYSTEMD,
            },
            TroubleKind::Forbidden => &[
                "The socket is root-only, by design.",
                "Open the console as the same user the agent runs as:  sudo vigil ui",
            ],
            TroubleKind::Unreadable => &[
                "Version mismatch between the agent and this console.",
                "Both ship in one package: install them together.",
            ],
        }
    }
}

impl fmt::Display for Trouble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.headline(), self.cause)
    }
}

impl std::error::Error for Trouble {}

#[cfg(test)]
mod tests {
    use vigil_config::{Installation, Service};

    use super::*;

    #[test]
    fn an_agent_that_is_not_answering_is_looked_for_with_the_service_manager_of_this_system() {
        assert!(ABSENT_UNDER_SYSTEMD[0].ends_with(Service::SYSTEMD.status));
        assert!(ABSENT_UNDER_LAUNCHD[0].ends_with(Service::LAUNCHD.status));

        let said = Trouble::new("/x", TroubleKind::Absent, "gone").what_to_try();
        assert!(
            said[0].ends_with(Installation::here().service.status),
            "a Mac has no systemctl, and a hint that names one sends the reader nowhere: {said:?}"
        );
    }
}
