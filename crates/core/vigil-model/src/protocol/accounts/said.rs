use super::account_change::AccountChange;

impl AccountChange {
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
