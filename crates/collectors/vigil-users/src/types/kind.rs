#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Account,
    Group,
    Sudoer,
    Key,
    Session,
    SessionSource,
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
            "session-source" => Kind::SessionSource,
            _ => Kind::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_saying_where_logins_are_read_from_is_not_a_person_logged_in() {
        assert_eq!(Kind::of("session|deploy|pts/0"), Kind::Session);
        assert_eq!(Kind::of("session-source|logind"), Kind::SessionSource);
        assert_ne!(
            Kind::of("session-source|logind"),
            Kind::Unknown,
            "a row this console does know must not open the view for rows it does not"
        );
    }
}
