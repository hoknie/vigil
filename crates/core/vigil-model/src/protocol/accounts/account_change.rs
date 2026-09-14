use serde::{Deserialize, Serialize};

use super::account_object::AccountObject;
use super::changing::Changing;

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
}
