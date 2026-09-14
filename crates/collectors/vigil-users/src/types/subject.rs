use vigil_model::{AccountObject, Snapshot};

use super::kind::Kind;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Subject {
    #[default]
    Users,
    Groups,
    Sudo,
    Keys,
    SshUsers,
    LoggedIn,
    Other,
}

impl Subject {
    pub const ALL: &'static [Subject] = &[
        Subject::Users,
        Subject::Groups,
        Subject::Sudo,
        Subject::Keys,
        Subject::SshUsers,
        Subject::LoggedIn,
        Subject::Other,
    ];

    pub fn object(self) -> Option<AccountObject> {
        match self {
            Subject::Users => Some(AccountObject::User),
            Subject::Groups => Some(AccountObject::Group),
            Subject::Sudo => Some(AccountObject::Sudo),
            Subject::Keys => Some(AccountObject::Key),
            Subject::SshUsers => Some(AccountObject::SshUser),
            Subject::LoggedIn => Some(AccountObject::Session),
            Subject::Other => None,
        }
    }

    pub fn shown(self, reading: &Snapshot) -> bool {
        self != Subject::Other || holds_something_unknown(reading)
    }

    pub fn name(self) -> &'static str {
        match self {
            Subject::Users => "users",
            Subject::Groups => "groups",
            Subject::Sudo => "sudo",
            Subject::Keys => "keys",
            Subject::SshUsers => "ssh users",
            Subject::LoggedIn => "logged in",
            Subject::Other => "other",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Subject::Users => "ACCOUNTS",
            Subject::Groups => "GROUPS",
            Subject::Sudo => "SUDO",
            Subject::Keys => "KEYS",
            Subject::SshUsers => "SSH USERS",
            Subject::LoggedIn => "LOGGED IN",
            Subject::Other => "OTHER",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Subject::Users | Subject::SshUsers => "THE SELECTED ACCOUNT",
            Subject::Groups => "THE SELECTED GROUP",
            Subject::Sudo => "THE SELECTED GRANT",
            Subject::Keys => "THE SELECTED KEY",
            Subject::LoggedIn => "THE SELECTED SESSION",
            Subject::Other => "THE SELECTED OBJECT",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Subject::Users => "every account this host will let in, and as whom",
            Subject::Groups => "every group, and what being in one grants",
            Subject::Sudo => "every grant in /etc/sudoers and /etc/sudoers.d: who may run what",
            Subject::Keys => "one row per key in an authorized_keys file the agent found",
            Subject::SshUsers => {
                "accounts with an authorized_keys file: let in by a key, not a password \
                 (what sshd allows is not in this reading)"
            }
            Subject::LoggedIn => {
                "who is logged in right now, from every one of this host's login records, \
                 and which of those records answered"
            }
            Subject::Other => "objects of a kind this console does not know, from a newer agent",
        }
    }

    pub fn kinds(self) -> &'static [Kind] {
        match self {
            Subject::Users => &[Kind::Account],
            Subject::Groups => &[Kind::Group],
            Subject::Sudo => &[Kind::Sudoer],
            Subject::Keys => &[Kind::Key],
            Subject::LoggedIn => &[Kind::Session, Kind::SessionSource],
            Subject::Other => &[Kind::Unknown],
            Subject::SshUsers => &[],
        }
    }

    pub fn thing(self) -> &'static str {
        match self {
            Subject::Users => "account",
            Subject::Groups => "group",
            Subject::Sudo => "sudo grant",
            Subject::Keys => "key",
            Subject::SshUsers => "account with a key file",
            Subject::LoggedIn => "session",
            Subject::Other => "object of an unknown kind",
        }
    }

    pub fn things(self, how_many: usize) -> String {
        if how_many == 1 {
            return self.thing().to_string();
        }
        match self {
            Subject::Users => "accounts".to_string(),
            Subject::Groups => "groups".to_string(),
            Subject::Sudo => "sudo grants".to_string(),
            Subject::Keys => "keys".to_string(),
            Subject::SshUsers => "accounts with a key file".to_string(),
            Subject::LoggedIn => "sessions".to_string(),
            Subject::Other => "objects of a kind this console does not know".to_string(),
        }
    }
}

fn holds_something_unknown(reading: &Snapshot) -> bool {
    reading
        .items
        .keys()
        .any(|key| Kind::of(key) == Kind::Unknown)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::Snapshot;

    use super::*;
    use crate::fixture::users;

    #[test]
    fn the_six_are_always_there_and_the_seventh_only_when_it_has_something_in_it() {
        let reading = users();

        assert_eq!(
            Subject::ALL
                .iter()
                .filter(|subject| subject.shown(&reading))
                .map(|subject| subject.name())
                .collect::<Vec<_>>(),
            vec!["users", "groups", "sudo", "keys", "ssh users", "logged in"],
            "a tab that opens onto nothing teaches people to stop pressing the tabs"
        );
    }

    #[test]
    fn a_kind_from_a_newer_agent_puts_a_view_on_the_row_to_hold_it() {
        let reading = Snapshot::new("users", "2026-09-13T09:00:00.000Z".to_string())
            .with("keyring|root|0x1234", json!({"from": "a newer agent"}));

        assert!(
            Subject::Other.shown(&reading),
            "an object of a kind this build never heard of is still shown to the reader"
        );
    }

    #[test]
    fn every_subject_says_what_it_holds_and_what_one_row_of_it_is() {
        for subject in Subject::ALL {
            assert!(!subject.name().is_empty());
            assert!(!subject.caption().is_empty());
            assert!(!subject.detail().is_empty());
            assert!(!subject.about().is_empty());
            assert!(!subject.thing().is_empty());
            assert_ne!(subject.things(2), subject.thing());
        }
    }
}
