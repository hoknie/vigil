use std::collections::BTreeSet;

pub const UTMP: &str = "utmp";

pub const LOGIND: &str = "logind";

const UNATTENDED_CLASSES: &[&str] = &[
    "background",
    "background-light",
    "manager",
    "manager-early",
    "none",
];

const CLOSING: &str = "closing";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    pub user: String,
    pub uid: Option<u32>,
    pub line: String,
    pub from: String,
    pub remote: bool,
    pub pid: u32,
    pub id: String,
    pub service: String,
    pub kind: String,
    pub class: String,
    pub state: String,
    pub sources: BTreeSet<&'static str>,
}

impl Session {
    pub fn is_remote(&self) -> bool {
        self.remote || !self.from.is_empty()
    }

    pub fn attended(&self) -> bool {
        self.state != CLOSING && !UNATTENDED_CLASSES.contains(&self.class.as_str())
    }

    pub fn key(&self) -> String {
        format!("session|{}|{}", self.who(), self.place())
    }

    pub fn who(&self) -> String {
        match (self.user.is_empty(), self.uid) {
            (false, _) => self.user.clone(),
            (true, Some(uid)) => format!("uid {uid}"),
            (true, None) => "unknown".to_string(),
        }
    }

    pub fn place(&self) -> String {
        if !self.line.is_empty() {
            return self.line.clone();
        }
        if !self.id.is_empty() {
            return format!("logind:{}", self.id);
        }
        if self.pid != 0 {
            return format!("pid:{}", self.pid);
        }
        "no terminal".to_string()
    }

    pub fn seen_by(&self) -> Vec<&'static str> {
        self.sources.iter().copied().collect()
    }
}
