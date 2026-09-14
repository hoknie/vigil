use serde::{Deserialize, Serialize};

use super::changing::Changing;

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
