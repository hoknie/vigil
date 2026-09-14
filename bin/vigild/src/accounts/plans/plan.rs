use vigil_model::{AccountChange, Snapshot};

use super::key_steps::{added, file_of_key, no_longer, owner, written};
use super::keys;
use super::reader::{Reader, present};
use super::sudo::sudo;
use crate::accounts::fields::{item, key_rows, number, sources, text};
use crate::accounts::step::Step;
use crate::accounts::utility::Utility;

pub fn plan(
    change: &AccountChange,
    reading: &Snapshot,
    read: Reader<'_>,
) -> Result<Vec<Step>, String> {
    match change {
        AccountChange::UpdateUser {
            name,
            shell,
            home,
            comment,
            locked,
            groups,
        } => {
            let mut arguments: Vec<String> = Vec::new();
            for (flag, value) in [("-s", shell), ("-d", home), ("-c", comment)] {
                if let Some(value) = value {
                    arguments.push(flag.to_string());
                    arguments.push(value.clone());
                }
            }
            match locked {
                Some(true) => arguments.push("-L".to_string()),
                Some(false) => arguments.push("-U".to_string()),
                None => {}
            }
            if let Some(groups) = groups {
                arguments.push("-G".to_string());
                arguments.push(groups.join(","));
            }
            arguments.push(name.clone());
            Ok(vec![run(Utility::Usermod, arguments)])
        }
        AccountChange::DeleteUser { name } => Ok(vec![run(Utility::Userdel, vec![name.clone()])]),
        AccountChange::CreateGroup { name, members } => {
            let mut steps = vec![run(Utility::Groupadd, vec![name.clone()])];
            if !members.is_empty() {
                steps.push(members_of(name, members));
            }
            Ok(steps)
        }
        AccountChange::UpdateGroup {
            name,
            rename,
            members,
        } => {
            let mut steps = Vec::new();
            if let Some(members) = members {
                steps.push(members_of(name, members));
            }
            if let Some(rename) = rename {
                steps.push(run(
                    Utility::Groupmod,
                    vec!["-n".to_string(), rename.clone(), name.clone()],
                ));
            }
            Ok(steps)
        }
        AccountChange::DeleteGroup { name } => Ok(vec![run(Utility::Groupdel, vec![name.clone()])]),
        AccountChange::UpdateSudo { who, rules } => sudo(reading, who, rules, read),
        AccountChange::DeleteSudo { who } => sudo(reading, who, &[], read),
        AccountChange::CreateKey { user, line } | AccountChange::CreateSshUser { user, line } => {
            added(reading, user, std::slice::from_ref(line), read)
        }
        AccountChange::UpdateKey {
            user,
            fingerprint,
            options,
            comment,
        } => {
            let (path, uid, gid) = file_of_key(reading, user, fingerprint)?;
            let text = present(&path, Some(uid), read)?;
            let after = keys::restyled(&text, fingerprint, options.as_deref(), comment.as_deref())
                .ok_or_else(|| no_longer(&path, fingerprint))?;
            Ok(vec![written(path, after, uid, gid)])
        }
        AccountChange::DeleteKey { user, fingerprint } => {
            let (path, uid, gid) = file_of_key(reading, user, fingerprint)?;
            let text = present(&path, Some(uid), read)?;
            let (after, taken) = keys::without(&text, std::slice::from_ref(fingerprint));
            if taken == 0 {
                return Err(no_longer(&path, fingerprint));
            }
            Ok(vec![written(path, after, uid, gid)])
        }
        AccountChange::UpdateSshUser {
            user,
            removed,
            added: lines,
        } => {
            let (uid, gid, _) = owner(reading, user)?;
            let rows = key_rows(reading, user);
            let paths = sources(&rows);
            let mut steps = Vec::new();
            let mut taken = 0;
            for (at, path) in paths.iter().enumerate() {
                let text = present(path, Some(uid), read)?;
                let (mut after, count) = keys::without(&text, removed);
                taken += count;
                if at == 0 && !lines.is_empty() {
                    after = keys::with(&after, lines);
                }
                if after != text {
                    steps.push(written(path.to_string(), after, uid, gid));
                }
            }
            if taken < removed.len() {
                return Err(format!(
                    "the key files of {user} no longer hold every key asked to be taken away: \
                     the reading is older than the files, and nothing was changed"
                ));
            }
            if paths.is_empty() && !lines.is_empty() {
                return added(reading, user, lines, read);
            }
            Ok(steps)
        }
        AccountChange::DeleteSshUser { user } => {
            let (uid, _, _) = owner(reading, user)?;
            Ok(sources(&key_rows(reading, user))
                .into_iter()
                .map(|path| Step::Remove {
                    path: path.to_string(),
                    holder: Some(uid),
                })
                .collect())
        }
        AccountChange::DeleteSession { key } => {
            let row = item(reading, key).ok_or_else(|| format!("{key} is no longer read"))?;
            match text(row, "session_id").filter(|id| !id.is_empty()) {
                Some(id) => Ok(vec![run(
                    Utility::Loginctl,
                    vec!["terminate-session".to_string(), id.to_string()],
                )]),
                None => Ok(vec![Step::Signal {
                    pid: number(row, "pid")
                        .and_then(|pid| u32::try_from(pid).ok())
                        .ok_or_else(|| format!("{key} carries no pid"))?,
                    uid: number(row, "uid").and_then(|uid| u32::try_from(uid).ok()),
                }]),
            }
        }
    }
}

fn run(utility: Utility, arguments: Vec<String>) -> Step {
    Step::Run { utility, arguments }
}

fn members_of(name: &str, members: &[String]) -> Step {
    run(
        Utility::Gpasswd,
        vec!["-M".to_string(), members.join(","), name.to_string()],
    )
}
