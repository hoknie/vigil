use serde_json::Value;
use vigil_model::{AccountChange, Snapshot};

use super::checks;
use super::fields::{flag, item, key_rows, number, sudoers_d_sources, text};
use super::ours::Ours;

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

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_users::fixture::users;

    use super::*;

    const OTHER: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRq ci@build";

    fn ours() -> Ours {
        Ours { uid: 998, pid: 77 }
    }

    fn refused(change: AccountChange) -> String {
        aim(&change, &users(), ours()).expect_err("refused")
    }

    fn allowed(change: AccountChange) {
        if let Err(why) = aim(&change, &users(), ours()) {
            panic!("{change:?} was refused: {why}");
        }
    }

    fn deploy_key() -> String {
        users()
            .items
            .keys()
            .find(|key| key.starts_with("sshkey|deploy|"))
            .and_then(|key| key.rsplit('|').next())
            .expect("deploy has a key")
            .to_string()
    }

    fn update_user(
        name: &str,
        shell: Option<&str>,
        locked: Option<bool>,
        groups: Option<Vec<String>>,
    ) -> AccountChange {
        AccountChange::UpdateUser {
            name: name.into(),
            shell: shell.map(String::from),
            home: None,
            comment: None,
            locked,
            groups,
        }
    }

    #[test]
    fn the_superuser_is_neither_deleted_nor_locked_whatever_the_console_asks() {
        assert!(
            refused(AccountChange::DeleteUser {
                name: "root".into()
            })
            .contains("uid 0")
        );
        assert!(refused(update_user("root", None, Some(true), None)).contains("uid 0"));
        allowed(update_user("root", Some("/bin/sh"), None, None));
    }

    #[test]
    fn the_account_the_agent_runs_as_is_not_deleted_on_request() {
        assert!(
            refused(AccountChange::DeleteUser {
                name: "svc-runner".into()
            })
            .contains("runs as")
        );
        allowed(AccountChange::DeleteUser {
            name: "contractor".into(),
        });
    }

    #[test]
    fn an_account_or_group_no_longer_in_the_reading_is_refused_by_name() {
        assert!(
            refused(AccountChange::DeleteUser {
                name: "mallory".into()
            })
            .contains("mallory")
        );
        assert!(
            refused(AccountChange::DeleteGroup {
                name: "docker".into()
            })
            .contains("docker")
        );
        assert!(
            refused(AccountChange::CreateGroup {
                name: "wheel".into(),
                members: Vec::new()
            })
            .contains("already")
        );
    }

    #[test]
    fn an_update_that_changes_nothing_is_refused_rather_than_run() {
        assert!(refused(update_user("deploy", None, None, None)).contains("nothing to change"));
    }

    #[test]
    fn what_goes_to_the_account_tools_is_checked_before_any_of_it_is_run() {
        assert!(refused(update_user("deploy", Some("bash"), None, None)).contains("absolute"));
        assert!(
            refused(update_user(
                "deploy",
                None,
                None,
                Some(vec!["docker".into()])
            ))
            .contains("docker")
        );
        assert!(
            refused(AccountChange::CreateGroup {
                name: "-g0".into(),
                members: Vec::new()
            })
            .contains("name")
        );
    }

    #[test]
    fn the_group_of_the_superuser_is_not_deleted() {
        assert!(
            refused(AccountChange::DeleteGroup {
                name: "root".into()
            })
            .contains("gid 0")
        );
    }

    #[test]
    fn a_grant_that_lives_only_in_etc_sudoers_is_left_for_a_person_with_visudo() {
        let why = refused(AccountChange::DeleteSudo {
            who: "%wheel".into(),
        });

        assert!(why.contains("/etc/sudoers"), "{why}");
    }

    #[test]
    fn a_grant_in_sudoers_d_is_rewritten_only_with_rules_and_never_emptied_by_an_update() {
        let mut reading = users();
        reading.items.insert(
            "sudoer|deploy".into(),
            json!({"who": "deploy", "rules": [{"source": "/etc/sudoers.d/deploy", "spec": "ALL=(ALL) ALL"}]}),
        );

        assert!(
            aim(
                &AccountChange::UpdateSudo {
                    who: "deploy".into(),
                    rules: vec!["ALL=(root) /usr/bin/id".into()]
                },
                &reading,
                ours()
            )
            .is_ok()
        );
        let emptied = aim(
            &AccountChange::UpdateSudo {
                who: "deploy".into(),
                rules: Vec::new(),
            },
            &reading,
            ours(),
        )
        .expect_err("an empty grant is a delete");
        assert!(emptied.contains("delete"), "{emptied}");
    }

    #[test]
    fn a_key_file_the_agent_could_not_read_is_not_changed_blind() {
        let mut reading = users();
        reading.items.insert(
            "account|backup".into(),
            json!({"name": "backup", "uid": 1001, "gid": 1001, "home": "/home/backup"}),
        );

        let why = aim(
            &AccountChange::DeleteSshUser {
                user: "backup".into(),
            },
            &reading,
            ours(),
        )
        .expect_err("refused");

        assert!(why.contains("could not be read"), "{why}");
    }

    #[test]
    fn a_key_already_letting_the_account_in_is_not_added_twice_and_a_new_one_is() {
        let deploy = users().items[&format!("sshkey|deploy|{}", deploy_key())].clone();
        let line = format!(
            "{} {}",
            "ssh-ed25519", "AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr"
        );
        assert!(flag(&deploy, "readable"));

        assert!(
            refused(AccountChange::CreateKey {
                user: "deploy".into(),
                line
            })
            .contains("already")
        );
        allowed(AccountChange::CreateKey {
            user: "deploy".into(),
            line: OTHER.into(),
        });
        assert!(
            refused(AccountChange::CreateSshUser {
                user: "deploy".into(),
                line: OTHER.into()
            })
            .contains("already")
        );
        allowed(AccountChange::CreateSshUser {
            user: "www-data".into(),
            line: OTHER.into(),
        });
    }

    #[test]
    fn a_session_is_ended_by_its_logind_id_or_its_pid_and_never_when_it_is_pid_1() {
        allowed(AccountChange::DeleteSession {
            key: "session|deploy|pts/0".into(),
        });
        assert!(
            refused(AccountChange::DeleteSession {
                key: "session-source|logind".into()
            })
            .contains("not a session")
        );

        let mut reading = users();
        reading.items.insert(
            "session|root|pid:1".into(),
            json!({"user": "root", "pid": 1, "session_id": ""}),
        );
        assert!(
            aim(
                &AccountChange::DeleteSession {
                    key: "session|root|pid:1".into()
                },
                &reading,
                ours()
            )
            .expect_err("pid 1")
            .contains("pid 1")
        );
    }
}
