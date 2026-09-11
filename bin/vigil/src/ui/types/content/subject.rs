use crate::ui::helpers::motion::step_along::step_along;
use crate::ui::screens::accounts::Kind;
use crate::ui::{Choice, Reading, View};

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

    pub fn on(view: &View) -> Vec<Subject> {
        Subject::ALL
            .iter()
            .copied()
            .filter(|subject| *subject != Subject::Other || holds_something_unknown(view))
            .collect()
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

    pub fn holding(key: &str) -> Subject {
        match Kind::of(key) {
            Kind::Account => Subject::Users,
            Kind::Group => Subject::Groups,
            Kind::Sudoer => Subject::Sudo,
            Kind::Key => Subject::Keys,
            Kind::Session | Kind::SessionSource => Subject::LoggedIn,
            Kind::Unknown => Subject::Other,
        }
    }
}

impl Choice for Subject {
    const COUNT: usize = Subject::ALL.len();

    fn index(self) -> usize {
        Subject::ALL
            .iter()
            .position(|subject| *subject == self)
            .unwrap_or(0)
    }

    fn step(self, by: isize, shown: &[Subject]) -> Subject {
        step_along(self, by, shown)
    }
}

fn holds_something_unknown(view: &View) -> bool {
    match view.reading("users") {
        Reading::Taken(snapshot) => snapshot
            .items
            .keys()
            .any(|key| Kind::of(key) == Kind::Unknown),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::Snapshot;

    use super::*;
    use crate::ui::fixture;

    #[test]
    fn the_six_are_always_there_and_the_seventh_only_when_it_has_something_in_it() {
        let shown = Subject::on(&fixture::view());

        assert_eq!(
            shown
                .iter()
                .map(|subject| subject.name())
                .collect::<Vec<_>>(),
            vec!["users", "groups", "sudo", "keys", "ssh users", "logged in"],
            "a tab that opens onto nothing teaches people to stop pressing the tabs"
        );
    }

    #[test]
    fn a_kind_from_a_newer_agent_puts_a_view_on_the_row_to_hold_it() {
        let mut view = fixture::view();
        view.readings.put(
            "users",
            Reading::Taken(
                Snapshot::new("users", "2026-09-09T09:00:00.000Z")
                    .with("keyring|root|0x1234", json!({"from": "a newer agent"})),
            ),
        );

        assert!(Subject::on(&view).contains(&Subject::Other));
    }

    #[test]
    fn every_kind_of_object_has_a_subject_that_holds_it() {
        assert_eq!(Subject::holding("account|deploy"), Subject::Users);
        assert_eq!(Subject::holding("group|docker"), Subject::Groups);
        assert_eq!(Subject::holding("sudoer|%wheel"), Subject::Sudo);
        assert_eq!(Subject::holding("sshkey|deploy|SHA256:x"), Subject::Keys);
        assert_eq!(Subject::holding("session|deploy|pts/0"), Subject::LoggedIn);
        assert_eq!(Subject::holding("keyring|root|0x1"), Subject::Other);
    }

    #[test]
    fn stepping_along_the_row_wraps_and_stays_on_what_is_shown() {
        let shown = Subject::on(&fixture::view());

        assert_eq!(Subject::Users.step(-1, &shown), Subject::LoggedIn);
        assert_eq!(Subject::LoggedIn.step(1, &shown), Subject::Users);
        assert_eq!(Subject::Users.step(1, &shown), Subject::Groups);
        assert!(
            !shown.contains(&Subject::Other),
            "and never onto a view that is not on the row"
        );
    }

    #[test]
    fn each_one_says_what_it_is_and_the_derived_one_says_what_it_is_not() {
        for subject in Subject::ALL {
            assert!(!subject.about().is_empty(), "{}", subject.name());
            assert!(!subject.caption().is_empty(), "{}", subject.name());
            assert!(!subject.detail().is_empty(), "{}", subject.name());
        }
        assert!(Subject::SshUsers.about().contains("authorized_keys"));
        assert!(Subject::SshUsers.about().contains("sshd"));
        assert!(Subject::SshUsers.kinds().is_empty());
    }
}
