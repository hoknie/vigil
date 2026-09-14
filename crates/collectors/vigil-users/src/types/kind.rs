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
    pub const KNOWN: &'static [Kind] = &[
        Kind::Account,
        Kind::Group,
        Kind::Sudoer,
        Kind::Key,
        Kind::Session,
        Kind::SessionSource,
    ];

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

    pub fn prefix(self) -> &'static str {
        match self {
            Kind::Account => "account|",
            Kind::Group => "group|",
            Kind::Sudoer => "sudoer|",
            Kind::Key => "sshkey|",
            Kind::Session => "session|",
            Kind::SessionSource => "session-source|",
            Kind::Unknown => "",
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

    #[test]
    fn every_known_kind_is_found_by_the_front_of_its_key_and_by_nothing_else() {
        for kind in Kind::KNOWN.iter().copied() {
            assert_eq!(Kind::of(&format!("{}anything", kind.prefix())), kind);
            assert!(kind.prefix().ends_with('|'));
        }
        assert!(
            !"session-source|logind".starts_with(Kind::Session.prefix()),
            "a login source must not be picked up as a session because both names start alike"
        );
    }
}
