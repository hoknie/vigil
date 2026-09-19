use std::collections::BTreeMap;

use vigil_collect::PasswdEntry;

use super::group::GroupEntry;
use super::shadow::{PasswordState, ShadowFacts};

const HELD_ELSEWHERE: &str = "********";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryAccount {
    pub entry: PasswdEntry,
    pub password: String,
}

pub fn password_state_of(field: &str) -> PasswordState {
    match field {
        "" => PasswordState::Empty,
        HELD_ELSEWHERE => PasswordState::Set,
        field if field.starts_with('*') => PasswordState::Disabled,
        field if field.starts_with('!') => PasswordState::Locked,
        _ => PasswordState::Set,
    }
}

pub fn merged_accounts(
    found: Vec<DirectoryAccount>,
) -> (Vec<PasswdEntry>, BTreeMap<String, ShadowFacts>) {
    let mut entries: Vec<PasswdEntry> = Vec::new();
    let mut facts = BTreeMap::new();

    for account in found {
        if facts.contains_key(&account.entry.name) {
            continue;
        }
        facts.insert(
            account.entry.name.clone(),
            ShadowFacts {
                password: password_state_of(&account.password),
                last_change_day: None,
                max_age_days: None,
                expires_day: None,
            },
        );
        entries.push(account.entry);
    }

    (entries, facts)
}

pub fn merged_groups(found: Vec<GroupEntry>) -> Vec<GroupEntry> {
    let mut groups: Vec<GroupEntry> = Vec::new();

    for group in found {
        match groups.iter_mut().find(|held| held.name == group.name) {
            Some(held) => {
                for member in group.members {
                    if !held.members.contains(&member) {
                        held.members.push(member);
                    }
                }
            }
            None => groups.push(group),
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(name: &str, uid: u32, password: &str) -> DirectoryAccount {
        DirectoryAccount {
            entry: PasswdEntry {
                name: name.into(),
                uid,
                gid: 20,
                home: format!("/Users/{name}"),
                shell: "/bin/zsh".into(),
            },
            password: password.into(),
        }
    }

    fn group(name: &str, gid: u32, members: &[&str]) -> GroupEntry {
        GroupEntry {
            name: name.into(),
            gid,
            members: members.iter().map(|member| member.to_string()).collect(),
        }
    }

    #[test]
    fn an_account_whose_password_is_kept_by_directory_services_has_one_set() {
        assert_eq!(password_state_of("********"), PasswordState::Set);
        assert!(password_state_of("********").permits_login());
    }

    #[test]
    fn an_account_with_a_star_for_a_password_takes_no_password_at_all() {
        assert_eq!(
            password_state_of("*"),
            PasswordState::Disabled,
            "root and every service account of a Mac are written this way, and root turned on \
             by dsenableroot is the move from this to a password that is set"
        );
        assert!(!password_state_of("*").permits_login());
        assert_eq!(password_state_of(""), PasswordState::Empty);
        assert_eq!(password_state_of("!"), PasswordState::Locked);
    }

    #[test]
    fn an_account_the_directory_and_the_flat_file_both_hold_is_read_once_as_found_first() {
        let (entries, facts) = merged_accounts(vec![
            account("root", 0, "*"),
            account("yclients", 501, "********"),
            account("root", 0, "********"),
        ]);

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["root", "yclients"],
            "macOS answers from its directory first and from /etc/passwd after it, and the \
             directory is what it logs in against"
        );
        assert_eq!(facts["root"].password, PasswordState::Disabled);
        assert_eq!(facts["yclients"].password, PasswordState::Set);
        assert_eq!(facts["root"].last_change_day, None);
    }

    #[test]
    fn a_group_both_sources_hold_is_one_group_with_the_members_of_both() {
        let groups = merged_groups(vec![
            group("admin", 80, &["root", "yclients"]),
            group("staff", 20, &["root"]),
            group("admin", 80, &["root", "intruder"]),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(
            groups[0].members,
            vec!["root", "yclients", "intruder"],
            "a member written only into /etc/group is still a member the day this Mac boots \
             into single-user mode, and a privileged group is read at its widest"
        );
    }

    #[test]
    fn nothing_a_password_was_written_as_reaches_the_facts() {
        let (_, facts) = merged_accounts(vec![account("old", 502, "$6$saltsalt$hashhashhash")]);

        let printed = format!("{facts:?}");
        assert!(
            !printed.contains("saltsalt") && !printed.contains("$6$"),
            "{printed}"
        );
        assert_eq!(facts["old"].password, PasswordState::Set);
    }
}
