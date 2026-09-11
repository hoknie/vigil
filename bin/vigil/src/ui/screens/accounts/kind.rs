#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Account,
    Group,
    Sudoer,
    Key,
    Session,
    Unknown,
}

impl Kind {
    pub fn of(key: &str) -> Kind {
        match key.split('|').next().unwrap_or_default() {
            "account" => Kind::Account,
            "group" => Kind::Group,
            "sudoer" => Kind::Sudoer,
            "sshkey" => Kind::Key,
            "session" => Kind::Session,
            _ => Kind::Unknown,
        }
    }
}
