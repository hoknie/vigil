use std::fmt;

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
            TroubleKind::Absent => &[
                "Is the agent running?  systemctl status vigild",
                "Is this the socket it opened?  the socket_path line of its configuration",
                "Point the console at another one:  vigil ui --socket /path/to/vigil.sock",
            ],
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
