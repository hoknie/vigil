use serde_json::Value;
use vigil_model::{AccountChange, Snapshot};

use super::checks;
use crate::accounts::fields::{flag, item, key_rows, number, sudoers_d_sources, text};
use crate::accounts::ours::Ours;

pub fn aim(change: &AccountChange, reading: &Snapshot, ours: Ours) -> Result<(), String> {
    if !change.object().offers(change.changing()) {
        return Err(format!(
            "a {} is not {}d from the console",
            change.object().named(),
            change.changing().as_str()
        ));
    }
    if change.is_empty() {
        return Err(
            "nothing to change was named: every field was left as the reading has it".to_string(),
        );
    }

    match change {
        AccountChange::UpdateUser {
            name,
            shell,
            home,
            comment,
            locked,
            groups,
        } => {
            let account = account(reading, name)?;
            if let Some(shell) = shell {
                checks::path("shell", shell)?;
            }
            if let Some(home) = home {
                checks::path("home directory", home)?;
            }
            if let Some(comment) = comment {
                checks::text("comment", comment)?;
            }
            if *locked == Some(true) && number(account, "uid") == Some(0) {
                return Err(format!(
                    "{name} is uid 0: locking it locks the one account every recovery of this \
                     host goes through, and this agent does not do that"
                ));
            }
            for group in groups.iter().flatten() {
                self::group(reading, group)?;
            }
            Ok(())
        }
        AccountChange::DeleteUser { name } => {
            let account = account(reading, name)?;
            let uid = number(account, "uid");
            if uid == Some(0) {
                return Err(format!(
                    "{name} is uid 0, and this agent does not delete the superuser"
                ));
            }
            if uid == Some(u64::from(ours.uid)) {
                return Err(format!(
                    "{name} is the account this agent runs as: it does not delete itself on \
                     request"
                ));
            }
            Ok(())
        }
        AccountChange::CreateGroup { name, members } => {
            checks::name("group", name)?;
            if item(reading, &format!("group|{name}")).is_some() {
                return Err(format!("there is a group {name} already"));
            }
            for member in members {
                account(reading, member)?;
            }
            Ok(())
        }
        AccountChange::UpdateGroup {
            name,
            rename,
            members,
        } => {
            group(reading, name)?;
            if let Some(rename) = rename {
                checks::name("group", rename)?;
                if item(reading, &format!("group|{rename}")).is_some() {
                    return Err(format!("there is a group {rename} already"));
                }
            }
            for member in members.iter().flatten() {
                account(reading, member)?;
            }
            Ok(())
        }
        AccountChange::DeleteGroup { name } => {
            let group = group(reading, name)?;
            match number(group, "gid") {
                Some(0) => Err(format!(
                    "{name} has gid 0, and this agent does not delete the superuser's group"
                )),
                _ => Ok(()),
            }
        }
        AccountChange::UpdateSudo { who, rules } => {
            grant(reading, who)?;
            if rules.is_empty() {
                return Err(format!(
                    "no rule is left for {who}: taking the whole grant away is a delete, and it \
                     is asked for as one"
                ));
            }
            for rule in rules {
                checks::rule(rule)?;
            }
            Ok(())
        }
        AccountChange::DeleteSudo { who } => grant(reading, who).map(|_| ()),
        AccountChange::CreateKey { user, line } => {
            account(reading, user)?;
            readable_keys(reading, user)?;
            let fingerprint = checks::key_line(line)?;
            match item(reading, &format!("sshkey|{user}|{fingerprint}")) {
                Some(_) => Err(format!("{user} is let in by this key already")),
                None => Ok(()),
            }
        }
        AccountChange::UpdateKey {
            user,
            fingerprint,
            options,
            comment,
        } => {
            key(reading, user, fingerprint)?;
            if let Some(options) = options {
                checks::one_line("options", options)?;
            }
            if let Some(comment) = comment {
                checks::one_line("comment", comment)?;
            }
            Ok(())
        }
        AccountChange::DeleteKey { user, fingerprint } => {
            key(reading, user, fingerprint).map(|_| ())
        }
        AccountChange::CreateSshUser { user, line } => {
            account(reading, user)?;
            let held = key_rows(reading, user).len();
            if held > 0 {
                return Err(format!(
                    "{user} is let in by a key already ({held} row(s) in the reading): another \
                     key is added in the list of keys"
                ));
            }
            checks::key_line(line).map(|_| ())
        }
        AccountChange::UpdateSshUser {
            user,
            removed,
            added,
        } => {
            account(reading, user)?;
            readable_keys(reading, user)?;
            for fingerprint in removed {
                key(reading, user, fingerprint)?;
            }
            for line in added {
                checks::key_line(line)?;
            }
            Ok(())
        }
        AccountChange::DeleteSshUser { user } => {
            account(reading, user)?;
            match readable_keys(reading, user)?.is_empty() {
                true => Err(format!(
                    "{user} is let in by no key: there is nothing to take"
                )),
                false => Ok(()),
            }
        }
        AccountChange::DeleteSession { key } => session(reading, key, ours).map(|_| ()),
    }
}

fn account<'a>(reading: &'a Snapshot, name: &str) -> Result<&'a Value, String> {
    checks::name("account", name)?;
    item(reading, &format!("account|{name}")).ok_or_else(|| {
        format!(
            "there is no account {name} in the reading the agent holds: it was removed, or it \
             was never there"
        )
    })
}

fn group<'a>(reading: &'a Snapshot, name: &str) -> Result<&'a Value, String> {
    checks::name("group", name)?;
    item(reading, &format!("group|{name}")).ok_or_else(|| {
        format!(
            "there is no group {name} in the reading the agent holds: it was removed, or it was \
             never there"
        )
    })
}

fn grant<'a>(reading: &'a Snapshot, who: &str) -> Result<&'a Value, String> {
    checks::principal(who)?;
    let grant = item(reading, &format!("sudoer|{who}"))
        .ok_or_else(|| format!("the reading the agent holds grants {who} no sudo"))?;
    match sudoers_d_sources(grant).is_empty() {
        true => Err(format!(
            "every rule of {who} is in /etc/sudoers, which is not changed from the console: a \
             mistake there costs this host sudo, and it is edited by hand with visudo"
        )),
        false => Ok(grant),
    }
}

fn readable_keys<'a>(
    reading: &'a Snapshot,
    user: &str,
) -> Result<Vec<(&'a str, &'a Value)>, String> {
    let rows = key_rows(reading, user);
    match rows.iter().any(|(_, row)| !flag(row, "readable")) {
        true => Err(format!(
            "the key file of {user} could not be read by the agent, so nothing in it is changed \
             blind"
        )),
        false => Ok(rows),
    }
}

fn key<'a>(reading: &'a Snapshot, user: &str, fingerprint: &str) -> Result<&'a Value, String> {
    checks::name("account", user)?;
    let row = item(reading, &format!("sshkey|{user}|{fingerprint}")).ok_or_else(|| {
        format!("the reading the agent holds lets {user} in by no key {fingerprint}")
    })?;
    match flag(row, "readable") {
        true => Ok(row),
        false => Err(format!(
            "the key file of {user} could not be read by the agent, so nothing in it is changed \
             blind"
        )),
    }
}

fn session<'a>(reading: &'a Snapshot, key: &str, ours: Ours) -> Result<&'a Value, String> {
    if !key.starts_with("session|") {
        return Err(format!("{key} is not a session"));
    }
    let row = item(reading, key)
        .ok_or_else(|| format!("{key} is no longer in the reading the agent holds"))?;
    let pid = number(row, "pid").filter(|pid| *pid > 0);
    if pid == Some(1) {
        return Err("that session is led by pid 1, and this agent does not signal it".to_string());
    }
    if pid == Some(u64::from(ours.pid)) {
        return Err("that session is led by this agent itself".to_string());
    }
    let named = text(row, "session_id").is_some_and(|id| !id.is_empty());
    match named || pid.is_some() {
        true => Ok(row),
        false => Err(format!(
            "{key} carries neither a logind session id nor a pid: there is nothing to end"
        )),
    }
}
