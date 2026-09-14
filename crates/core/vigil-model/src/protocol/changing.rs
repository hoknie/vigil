use serde::{Deserialize, Serialize};

use crate::Rfc3339;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Changing {
    Create,
    Update,
    Delete,
}

impl Changing {
    pub const ALL: &'static [Changing] = &[Changing::Create, Changing::Update, Changing::Delete];

    pub fn as_str(self) -> &'static str {
        match self {
            Changing::Create => "create",
            Changing::Update => "update",
            Changing::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountObject {
    User,
    Group,
    Sudo,
    Key,
    SshUser,
    Session,
}

impl AccountObject {
    pub const ALL: &'static [AccountObject] = &[
        AccountObject::User,
        AccountObject::Group,
        AccountObject::Sudo,
        AccountObject::Key,
        AccountObject::SshUser,
        AccountObject::Session,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            AccountObject::User => "user",
            AccountObject::Group => "group",
            AccountObject::Sudo => "sudo",
            AccountObject::Key => "key",
            AccountObject::SshUser => "ssh_user",
            AccountObject::Session => "session",
        }
    }

    pub fn changings(self) -> &'static [Changing] {
        match self {
            AccountObject::User | AccountObject::Sudo => &[Changing::Update, Changing::Delete],
            AccountObject::Group | AccountObject::Key | AccountObject::SshUser => Changing::ALL,
            AccountObject::Session => &[Changing::Delete],
        }
    }

    pub fn offers(self, changing: Changing) -> bool {
        self.changings().contains(&changing)
    }

    pub fn named(self) -> &'static str {
        match self {
            AccountObject::User => "account",
            AccountObject::Group => "group",
            AccountObject::Sudo => "sudo grant",
            AccountObject::Key => "key",
            AccountObject::SshUser => "account let in by a key",
            AccountObject::Session => "session",
        }
    }

    pub fn deleting(self) -> &'static str {
        match self {
            AccountObject::User => "delete the account; its home directory stays",
            AccountObject::Group => "delete the group",
            AccountObject::Sudo => "take the grant out of /etc/sudoers.d",
            AccountObject::Key => "take the key out of authorized_keys",
            AccountObject::SshUser => "take away every key this account is let in by",
            AccountObject::Session => "end the session",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum AccountChange {
    UpdateUser {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shell: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        home: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        comment: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        locked: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        groups: Option<Vec<String>>,
    },
    DeleteUser {
        name: String,
    },
    CreateGroup {
        name: String,
        #[serde(default)]
        members: Vec<String>,
    },
    UpdateGroup {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rename: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        members: Option<Vec<String>>,
    },
    DeleteGroup {
        name: String,
    },
    UpdateSudo {
        who: String,
        #[serde(default)]
        rules: Vec<String>,
    },
    DeleteSudo {
        who: String,
    },
    CreateKey {
        user: String,
        line: String,
    },
    UpdateKey {
        user: String,
        fingerprint: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        options: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        comment: Option<String>,
    },
    DeleteKey {
        user: String,
        fingerprint: String,
    },
    CreateSshUser {
        user: String,
        line: String,
    },
    UpdateSshUser {
        user: String,
        #[serde(default)]
        removed: Vec<String>,
        #[serde(default)]
        added: Vec<String>,
    },
    DeleteSshUser {
        user: String,
    },
    DeleteSession {
        key: String,
    },
}

impl AccountChange {
    pub fn object(&self) -> AccountObject {
        match self {
            AccountChange::UpdateUser { .. } | AccountChange::DeleteUser { .. } => {
                AccountObject::User
            }
            AccountChange::CreateGroup { .. }
            | AccountChange::UpdateGroup { .. }
            | AccountChange::DeleteGroup { .. } => AccountObject::Group,
            AccountChange::UpdateSudo { .. } | AccountChange::DeleteSudo { .. } => {
                AccountObject::Sudo
            }
            AccountChange::CreateKey { .. }
            | AccountChange::UpdateKey { .. }
            | AccountChange::DeleteKey { .. } => AccountObject::Key,
            AccountChange::CreateSshUser { .. }
            | AccountChange::UpdateSshUser { .. }
            | AccountChange::DeleteSshUser { .. } => AccountObject::SshUser,
            AccountChange::DeleteSession { .. } => AccountObject::Session,
        }
    }

    pub fn changing(&self) -> Changing {
        match self {
            AccountChange::CreateGroup { .. }
            | AccountChange::CreateKey { .. }
            | AccountChange::CreateSshUser { .. } => Changing::Create,
            AccountChange::UpdateUser { .. }
            | AccountChange::UpdateGroup { .. }
            | AccountChange::UpdateSudo { .. }
            | AccountChange::UpdateKey { .. }
            | AccountChange::UpdateSshUser { .. } => Changing::Update,
            AccountChange::DeleteUser { .. }
            | AccountChange::DeleteGroup { .. }
            | AccountChange::DeleteSudo { .. }
            | AccountChange::DeleteKey { .. }
            | AccountChange::DeleteSshUser { .. }
            | AccountChange::DeleteSession { .. } => Changing::Delete,
        }
    }

    pub fn key(&self) -> String {
        match self {
            AccountChange::UpdateUser { name, .. } | AccountChange::DeleteUser { name } => {
                format!("account|{name}")
            }
            AccountChange::CreateGroup { name, .. }
            | AccountChange::UpdateGroup { name, .. }
            | AccountChange::DeleteGroup { name } => format!("group|{name}"),
            AccountChange::UpdateSudo { who, .. } | AccountChange::DeleteSudo { who } => {
                format!("sudoer|{who}")
            }
            AccountChange::CreateKey { user, .. } => format!("sshkey|{user}"),
            AccountChange::UpdateKey {
                user, fingerprint, ..
            }
            | AccountChange::DeleteKey { user, fingerprint } => {
                format!("sshkey|{user}|{fingerprint}")
            }
            AccountChange::CreateSshUser { user, .. }
            | AccountChange::UpdateSshUser { user, .. }
            | AccountChange::DeleteSshUser { user } => format!("account|{user}"),
            AccountChange::DeleteSession { key } => key.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            AccountChange::UpdateUser {
                shell,
                home,
                comment,
                locked,
                groups,
                ..
            } => {
                shell.is_none()
                    && home.is_none()
                    && comment.is_none()
                    && locked.is_none()
                    && groups.is_none()
            }
            AccountChange::UpdateGroup {
                rename, members, ..
            } => rename.is_none() && members.is_none(),
            AccountChange::UpdateKey {
                options, comment, ..
            } => options.is_none() && comment.is_none(),
            AccountChange::UpdateSshUser { removed, added, .. } => {
                removed.is_empty() && added.is_empty()
            }
            _ => false,
        }
    }

    pub fn said(&self) -> String {
        match self {
            AccountChange::UpdateUser {
                name,
                shell,
                home,
                comment,
                locked,
                groups,
            } => {
                let mut parts: Vec<String> = Vec::new();
                if let Some(shell) = shell {
                    parts.push(format!("shell {shell}"));
                }
                if let Some(home) = home {
                    parts.push(format!("home {home}"));
                }
                if let Some(comment) = comment {
                    parts.push(format!("comment {comment:?}"));
                }
                match locked {
                    Some(true) => parts.push("lock it".to_string()),
                    Some(false) => parts.push("unlock it".to_string()),
                    None => {}
                }
                if let Some(groups) = groups {
                    parts.push(match groups.is_empty() {
                        true => "in no group but its own".to_string(),
                        false => format!("groups {}", groups.join(", ")),
                    });
                }
                match parts.is_empty() {
                    true => format!("change nothing on the account {name}"),
                    false => format!("change the account {name}: {}", parts.join(", ")),
                }
            }
            AccountChange::DeleteUser { name } => {
                format!("delete the account {name}, keeping its home directory")
            }
            AccountChange::CreateGroup { name, members } => match members.is_empty() {
                true => format!("create the group {name}"),
                false => format!("create the group {name} with {}", members.join(", ")),
            },
            AccountChange::UpdateGroup {
                name,
                rename,
                members,
            } => {
                let mut parts: Vec<String> = Vec::new();
                if let Some(rename) = rename {
                    parts.push(format!("rename it to {rename}"));
                }
                if let Some(members) = members {
                    parts.push(match members.is_empty() {
                        true => "no members".to_string(),
                        false => format!("members {}", members.join(", ")),
                    });
                }
                match parts.is_empty() {
                    true => format!("change nothing on the group {name}"),
                    false => format!("change the group {name}: {}", parts.join(", ")),
                }
            }
            AccountChange::DeleteGroup { name } => format!("delete the group {name}"),
            AccountChange::UpdateSudo { who, rules } => format!(
                "write the sudo grant of {who} in /etc/sudoers.d as {} rule(s)",
                rules.len()
            ),
            AccountChange::DeleteSudo { who } => {
                format!("take the sudo grant of {who} out of /etc/sudoers.d")
            }
            AccountChange::CreateKey { user, .. } => format!("let {user} in by one more key"),
            AccountChange::UpdateKey {
                user, fingerprint, ..
            } => format!("change the options or the comment of the key {fingerprint} of {user}"),
            AccountChange::DeleteKey { user, fingerprint } => {
                format!("take the key {fingerprint} away from {user}")
            }
            AccountChange::CreateSshUser { user, .. } => format!("let {user} in by a key"),
            AccountChange::UpdateSshUser {
                user,
                removed,
                added,
            } => format!(
                "take {} key(s) away from {user} and add {}",
                removed.len(),
                added.len()
            ),
            AccountChange::DeleteSshUser { user } => format!("take every key away from {user}"),
            AccountChange::DeleteSession { key } => format!("end the session {key}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changed {
    pub key: String,
    pub object: AccountObject,
    pub changing: Changing,
    pub done: bool,
    pub said: String,
}

impl Changed {
    pub fn done(change: &AccountChange, said: impl Into<String>) -> Changed {
        Changed {
            key: change.key(),
            object: change.object(),
            changing: change.changing(),
            done: true,
            said: said.into(),
        }
    }

    pub fn refused(change: &AccountChange, said: impl Into<String>) -> Changed {
        Changed {
            done: false,
            ..Changed::done(change, said)
        }
    }

    pub fn keyed(self, key: impl Into<String>) -> Changed {
        Changed {
            key: key.into(),
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeReport {
    pub acted_at: Rfc3339,
    pub changed: Vec<Changed>,
}

impl ChangeReport {
    pub fn done(&self) -> usize {
        self.changed.iter().filter(|one| one.done).count()
    }

    pub fn refused(&self) -> usize {
        self.changed.len() - self.done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_of_each() -> Vec<AccountChange> {
        vec![
            AccountChange::UpdateUser {
                name: "deploy".into(),
                shell: Some("/bin/sh".into()),
                home: None,
                comment: None,
                locked: Some(true),
                groups: None,
            },
            AccountChange::DeleteUser {
                name: "deploy".into(),
            },
            AccountChange::CreateGroup {
                name: "ops".into(),
                members: vec!["deploy".into()],
            },
            AccountChange::UpdateGroup {
                name: "ops".into(),
                rename: Some("operators".into()),
                members: None,
            },
            AccountChange::DeleteGroup { name: "ops".into() },
            AccountChange::UpdateSudo {
                who: "deploy".into(),
                rules: vec!["ALL=(ALL) ALL".into()],
            },
            AccountChange::DeleteSudo {
                who: "deploy".into(),
            },
            AccountChange::CreateKey {
                user: "deploy".into(),
                line: "ssh-ed25519 AAAA person@laptop".into(),
            },
            AccountChange::UpdateKey {
                user: "deploy".into(),
                fingerprint: "SHA256:abc".into(),
                options: Some("no-pty".into()),
                comment: None,
            },
            AccountChange::DeleteKey {
                user: "deploy".into(),
                fingerprint: "SHA256:abc".into(),
            },
            AccountChange::CreateSshUser {
                user: "contractor".into(),
                line: "ssh-ed25519 AAAA person@laptop".into(),
            },
            AccountChange::UpdateSshUser {
                user: "contractor".into(),
                removed: vec!["SHA256:abc".into()],
                added: Vec::new(),
            },
            AccountChange::DeleteSshUser {
                user: "contractor".into(),
            },
            AccountChange::DeleteSession {
                key: "session|deploy|pts/0".into(),
            },
        ]
    }

    #[test]
    fn what_each_list_of_accounts_can_have_done_to_it_is_the_table_the_owner_decided() {
        use Changing::{Create, Delete, Update};

        assert_eq!(AccountObject::User.changings(), &[Update, Delete]);
        assert_eq!(AccountObject::Group.changings(), &[Create, Update, Delete]);
        assert_eq!(AccountObject::Sudo.changings(), &[Update, Delete]);
        assert_eq!(AccountObject::Key.changings(), &[Create, Update, Delete]);
        assert_eq!(
            AccountObject::SshUser.changings(),
            &[Create, Update, Delete]
        );
        assert_eq!(
            AccountObject::Session.changings(),
            &[Delete],
            "a session is ended and nothing else: there is nothing in one to edit, and a \
             session is not created from a console"
        );
    }

    #[test]
    fn every_change_the_protocol_can_carry_is_one_its_object_offers() {
        let every = one_of_each();

        for change in &every {
            assert!(
                change.object().offers(change.changing()),
                "{change:?} is a change the table of what each list offers does not hold"
            );
        }
        for object in AccountObject::ALL {
            for changing in object.changings() {
                assert!(
                    every
                        .iter()
                        .any(|change| change.object() == *object && change.changing() == *changing),
                    "{} offers to {} and the protocol has no way to ask for it",
                    object.as_str(),
                    changing.as_str()
                );
            }
        }
    }

    #[test]
    fn a_change_round_trips_through_the_wire_under_the_name_of_what_it_does() {
        for change in one_of_each() {
            let line = serde_json::to_string(&change).expect("serialises");
            assert!(line.contains("\"change\":\""), "{line}");
            assert_eq!(
                serde_json::from_str::<AccountChange>(&line).expect("reads back"),
                change
            );
        }
        assert_eq!(
            serde_json::to_string(&AccountChange::DeleteUser {
                name: "deploy".into()
            })
            .expect("serialises"),
            "{\"change\":\"delete_user\",\"name\":\"deploy\"}"
        );
    }

    #[test]
    fn a_change_this_build_has_not_heard_of_is_refused_rather_than_guessed_at() {
        for line in [
            "{\"change\":\"create_user\",\"name\":\"eve\"}",
            "{\"change\":\"update_session\",\"key\":\"session|deploy|pts/0\"}",
            "{\"change\":\"run\",\"command\":\"id\"}",
        ] {
            assert!(
                serde_json::from_str::<AccountChange>(line).is_err(),
                "{line} is not in the table of what the console may ask for"
            );
        }
    }

    #[test]
    fn every_change_names_the_row_of_the_reading_it_is_about() {
        for change in one_of_each() {
            let key = change.key();
            let family = match change.object() {
                AccountObject::User | AccountObject::SshUser => "account|",
                AccountObject::Group => "group|",
                AccountObject::Sudo => "sudoer|",
                AccountObject::Key => "sshkey|",
                AccountObject::Session => "session|",
            };
            assert!(key.starts_with(family), "{change:?} names {key}");
            assert!(!change.said().is_empty());
        }
    }

    #[test]
    fn an_update_that_changes_nothing_says_so_and_a_delete_is_never_empty() {
        let nothing = AccountChange::UpdateUser {
            name: "deploy".into(),
            shell: None,
            home: None,
            comment: None,
            locked: None,
            groups: None,
        };

        assert!(nothing.is_empty());
        assert!(nothing.said().contains("nothing"), "{}", nothing.said());
        for change in one_of_each() {
            assert!(!change.is_empty(), "{change:?}");
        }
    }

    #[test]
    fn every_object_says_in_words_what_deleting_one_does() {
        for object in AccountObject::ALL {
            assert!(!object.deleting().is_empty());
            assert!(!object.named().is_empty());
        }
        assert!(
            AccountObject::User
                .deleting()
                .contains("home directory stays"),
            "the owner decided the home stays, and the band is where the reader learns that"
        );
    }

    #[test]
    fn a_report_names_every_row_the_agent_left_alone_and_counts_both_halves() {
        let every = one_of_each();
        let report = ChangeReport {
            acted_at: "2026-09-14T10:00:00.000Z".into(),
            changed: vec![
                Changed::done(&every[0], "usermod finished"),
                Changed::refused(&every[1], "uid 0 is not deleted"),
            ],
        };

        assert_eq!(report.done(), 1);
        assert_eq!(report.refused(), 1);
        assert_eq!(report.changed[1].key, "account|deploy");
        assert_eq!(report.changed[1].object, AccountObject::User);
        assert_eq!(report.changed[1].changing, Changing::Delete);
        assert_eq!(
            Changed::done(&every[7], "added")
                .keyed("sshkey|deploy|SHA256:new")
                .key,
            "sshkey|deploy|SHA256:new"
        );
    }
}
