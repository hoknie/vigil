use serde_json::Value;
use vigil_model::{AccountChange, Snapshot};

use super::fields::{flag, item, key_rows, number, sources, sudoers_d_sources, text};
use super::keys;
use super::step::Step;
use super::sudoers;
use super::utility::Utility;

pub type Reader<'a> = &'a dyn Fn(&str, Option<u32>) -> Result<Option<String>, String>;

const KEY_FILE_MODE: u32 = 0o600;

const KEY_DIRECTORY_MODE: u32 = 0o700;

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

fn written(path: String, text: String, uid: u32, gid: u32) -> Step {
    Step::Write {
        path,
        text,
        uid,
        gid,
        mode: KEY_FILE_MODE,
    }
}

fn sudo(
    reading: &Snapshot,
    who: &str,
    rules: &[String],
    read: Reader<'_>,
) -> Result<Vec<Step>, String> {
    let grant = item(reading, &format!("sudoer|{who}"))
        .ok_or_else(|| format!("the reading grants {who} no sudo"))?;
    let mut files: Vec<(&str, String)> = Vec::new();
    for path in sudoers_d_sources(grant) {
        files.push((path, present(path, None, read)?));
    }

    let Some(first) = files
        .iter()
        .position(|(_, text)| sudoers::without(text, who).1 > 0)
    else {
        return Err(format!(
            "no file in /etc/sudoers.d holds a rule for {who} any more: the reading is older \
             than the files, and nothing was changed"
        ));
    };

    Ok(files
        .iter()
        .enumerate()
        .filter_map(|(at, (path, text))| {
            let (after, found) = match at == first {
                true => sudoers::replacing(text, who, rules),
                false => sudoers::without(text, who),
            };
            if found == 0 {
                return None;
            }
            Some(match sudoers::holds_rules(&after) {
                true => Step::Sudoers {
                    path: path.to_string(),
                    text: after,
                },
                false => Step::Remove {
                    path: path.to_string(),
                    holder: None,
                },
            })
        })
        .collect())
}

fn added(
    reading: &Snapshot,
    user: &str,
    lines: &[String],
    read: Reader<'_>,
) -> Result<Vec<Step>, String> {
    let (uid, gid, home) = owner(reading, user)?;
    let rows = key_rows(reading, user);
    if let Some(path) = sources(&rows).first() {
        let text = present(path, Some(uid), read)?;
        return Ok(vec![written(
            path.to_string(),
            keys::with(&text, lines),
            uid,
            gid,
        )]);
    }

    let directory = format!("{}/.ssh", home.trim_end_matches('/'));
    let path = format!("{directory}/authorized_keys");
    let text = read(&path, Some(uid))?.unwrap_or_default();
    Ok(vec![
        Step::Directory {
            path: directory,
            uid,
            gid,
            mode: KEY_DIRECTORY_MODE,
        },
        written(path, keys::with(&text, lines), uid, gid),
    ])
}

fn owner<'a>(reading: &'a Snapshot, user: &str) -> Result<(u32, u32, &'a str), String> {
    let account = item(reading, &format!("account|{user}"))
        .ok_or_else(|| format!("there is no account {user} in the reading"))?;
    let home = text(account, "home").unwrap_or_default();
    if home.is_empty() || home == "/" || !home.starts_with('/') {
        return Err(format!(
            "{user} has no home directory a key file can be kept in ({home:?})"
        ));
    }
    Ok((id(account, "uid")?, id(account, "gid")?, home))
}

fn file_of_key(
    reading: &Snapshot,
    user: &str,
    fingerprint: &str,
) -> Result<(String, u32, u32), String> {
    let row = item(reading, &format!("sshkey|{user}|{fingerprint}"))
        .filter(|row| flag(row, "readable"))
        .ok_or_else(|| format!("the reading lets {user} in by no key {fingerprint}"))?;
    let path = text(row, "source")
        .ok_or_else(|| format!("the reading does not say which file holds {fingerprint}"))?;
    let (uid, gid, _) = owner(reading, user)?;
    Ok((path.to_string(), uid, gid))
}

fn present(path: &str, holder: Option<u32>, read: Reader<'_>) -> Result<String, String> {
    read(path, holder)?
        .ok_or_else(|| format!("{path} is no longer there: the reading is older than it"))
}

fn no_longer(path: &str, fingerprint: &str) -> String {
    format!("{path} no longer holds the key {fingerprint}: the reading is older than the file")
}

fn id(account: &Value, field: &str) -> Result<u32, String> {
    number(account, field)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("the reading carries no {field} for this account"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use vigil_users::fixture::users;

    use super::*;

    const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

    const OTHER: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRq ci@build";

    fn disk(files: &[(&str, &str)]) -> BTreeMap<String, String> {
        files
            .iter()
            .map(|(path, text)| (path.to_string(), text.to_string()))
            .collect()
    }

    fn planned(change: AccountChange, reading: &Snapshot, files: &[(&str, &str)]) -> Vec<Step> {
        let disk = disk(files);
        plan(&change, reading, &|path, _| Ok(disk.get(path).cloned())).expect("a plan")
    }

    fn fingerprint(line: &str) -> String {
        vigil_users::parse_authorized_keys(line)[0]
            .fingerprint
            .clone()
    }

    fn arguments(steps: &[Step]) -> Vec<(Utility, Vec<String>)> {
        steps
            .iter()
            .filter_map(|step| match step {
                Step::Run { utility, arguments } => Some((*utility, arguments.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn an_account_is_changed_by_one_usermod_with_each_value_its_own_argument_and_the_name_last() {
        let steps = planned(
            AccountChange::UpdateUser {
                name: "deploy".into(),
                shell: Some("/bin/sh".into()),
                home: Some("/srv/deploy".into()),
                comment: Some("Deploy; rm -rf /".into()),
                locked: Some(true),
                groups: Some(vec!["wheel".into(), "users".into()]),
            },
            &users(),
            &[],
        );

        assert_eq!(
            arguments(&steps),
            vec![(
                Utility::Usermod,
                [
                    "-s",
                    "/bin/sh",
                    "-d",
                    "/srv/deploy",
                    "-c",
                    "Deploy; rm -rf /",
                    "-L",
                    "-G",
                    "wheel,users",
                    "deploy"
                ]
                .map(String::from)
                .to_vec()
            )],
            "no -m: the home field changes, the files stay where they are; and a comment \
             that reads like a command is one argument that no shell ever sees"
        );
    }

    #[test]
    fn taking_every_supplementary_group_away_is_an_empty_list_and_not_a_missing_flag() {
        let steps = planned(
            AccountChange::UpdateUser {
                name: "deploy".into(),
                shell: None,
                home: None,
                comment: None,
                locked: Some(false),
                groups: Some(Vec::new()),
            },
            &users(),
            &[],
        );

        assert_eq!(
            arguments(&steps)[0].1,
            ["-U", "-G", "", "deploy"].map(String::from).to_vec()
        );
    }

    #[test]
    fn an_account_is_deleted_without_its_home_directory() {
        let steps = planned(
            AccountChange::DeleteUser {
                name: "contractor".into(),
            },
            &users(),
            &[],
        );

        assert_eq!(
            arguments(&steps),
            vec![(Utility::Userdel, vec!["contractor".to_string()])],
            "the owner decided the home stays: userdel without -r"
        );
    }

    #[test]
    fn a_group_is_created_then_given_its_members_and_renamed_only_after_they_are_set() {
        assert_eq!(
            arguments(&planned(
                AccountChange::CreateGroup {
                    name: "ops".into(),
                    members: vec!["deploy".into(), "contractor".into()]
                },
                &users(),
                &[]
            )),
            vec![
                (Utility::Groupadd, vec!["ops".to_string()]),
                (
                    Utility::Gpasswd,
                    ["-M", "deploy,contractor", "ops"]
                        .map(String::from)
                        .to_vec()
                )
            ]
        );
        assert_eq!(
            arguments(&planned(
                AccountChange::UpdateGroup {
                    name: "wheel".into(),
                    rename: Some("admins".into()),
                    members: Some(Vec::new())
                },
                &users(),
                &[]
            )),
            vec![
                (
                    Utility::Gpasswd,
                    ["-M", "", "wheel"].map(String::from).to_vec()
                ),
                (
                    Utility::Groupmod,
                    ["-n", "admins", "wheel"].map(String::from).to_vec()
                )
            ]
        );
    }

    fn with_sudoers_d() -> Snapshot {
        let mut reading = users();
        reading.items.insert(
            "sudoer|deploy".into(),
            json!({"who": "deploy", "rules": [
                {"source": "/etc/sudoers.d/deploy", "spec": "ALL=(ALL) ALL"},
                {"source": "/etc/sudoers.d/extra", "spec": "ALL=(root) /usr/bin/id"},
                {"source": "/etc/sudoers", "spec": "ALL=(ALL) ALL"}
            ]}),
        );
        reading
    }

    #[test]
    fn a_grant_rewritten_goes_through_visudo_in_the_first_file_that_holds_it_and_leaves_etc_sudoers_alone()
     {
        let steps = planned(
            AccountChange::UpdateSudo {
                who: "deploy".into(),
                rules: vec!["ALL=(root) /usr/bin/systemctl restart app".into()],
            },
            &with_sudoers_d(),
            &[
                ("/etc/sudoers.d/deploy", "# app\ndeploy ALL=(ALL) ALL\n"),
                (
                    "/etc/sudoers.d/extra",
                    "deploy ALL=(root) /usr/bin/id\n%ops ALL=(ALL) ALL\n",
                ),
            ],
        );

        assert_eq!(
            steps,
            vec![
                Step::Sudoers {
                    path: "/etc/sudoers.d/deploy".into(),
                    text: "# app\ndeploy ALL=(root) /usr/bin/systemctl restart app\n".into()
                },
                Step::Sudoers {
                    path: "/etc/sudoers.d/extra".into(),
                    text: "%ops ALL=(ALL) ALL\n".into()
                },
            ]
        );
    }

    #[test]
    fn a_grant_taken_away_removes_a_file_left_with_nothing_but_comments() {
        let steps = planned(
            AccountChange::DeleteSudo {
                who: "deploy".into(),
            },
            &with_sudoers_d(),
            &[
                ("/etc/sudoers.d/deploy", "# app\ndeploy ALL=(ALL) ALL\n"),
                (
                    "/etc/sudoers.d/extra",
                    "deploy ALL=(root) /usr/bin/id\n%ops ALL=(ALL) ALL\n",
                ),
            ],
        );

        assert_eq!(
            steps[0],
            Step::Remove {
                path: "/etc/sudoers.d/deploy".into(),
                holder: None
            }
        );
        assert!(matches!(&steps[1], Step::Sudoers { text, .. } if text == "%ops ALL=(ALL) ALL\n"));
    }

    #[test]
    fn files_that_no_longer_hold_the_grant_the_reading_saw_change_nothing() {
        let disk = disk(&[
            ("/etc/sudoers.d/deploy", "# emptied by hand\n"),
            ("/etc/sudoers.d/extra", "%ops ALL=(ALL) ALL\n"),
        ]);

        let complaint = plan(
            &AccountChange::DeleteSudo {
                who: "deploy".into(),
            },
            &with_sudoers_d(),
            &|path, _| Ok(disk.get(path).cloned()),
        )
        .expect_err("stale");

        assert!(complaint.contains("older than the files"), "{complaint}");
    }

    #[test]
    fn a_first_key_for_an_account_makes_its_ssh_directory_owned_by_it_and_a_private_file() {
        let steps = planned(
            AccountChange::CreateSshUser {
                user: "www-data".into(),
                line: OTHER.into(),
            },
            &users(),
            &[],
        );

        assert_eq!(
            steps,
            vec![
                Step::Directory {
                    path: "/var/www/.ssh".into(),
                    uid: 33,
                    gid: 33,
                    mode: 0o700
                },
                Step::Write {
                    path: "/var/www/.ssh/authorized_keys".into(),
                    text: format!("{OTHER}\n"),
                    uid: 33,
                    gid: 33,
                    mode: 0o600
                }
            ]
        );
    }

    #[test]
    fn another_key_goes_into_the_file_the_reading_found_the_account_s_keys_in() {
        let steps = planned(
            AccountChange::CreateKey {
                user: "deploy".into(),
                line: OTHER.into(),
            },
            &users(),
            &[("/home/deploy/.ssh/authorized_keys", &format!("{KEY}\n"))],
        );

        assert_eq!(
            steps,
            vec![Step::Write {
                path: "/home/deploy/.ssh/authorized_keys".into(),
                text: format!("{KEY}\n{OTHER}\n"),
                uid: 1000,
                gid: 1000,
                mode: 0o600
            }]
        );
    }

    #[test]
    fn a_key_taken_away_rewrites_its_file_and_a_key_already_gone_changes_nothing() {
        let reading = users();
        let path = "/home/deploy/.ssh/authorized_keys";
        let steps = planned(
            AccountChange::DeleteKey {
                user: "deploy".into(),
                fingerprint: fingerprint(KEY),
            },
            &reading,
            &[(path, &format!("{KEY}\n{OTHER}\n"))],
        );
        assert!(matches!(&steps[0], Step::Write { text, .. } if *text == format!("{OTHER}\n")));

        let disk = disk(&[(path, &format!("{OTHER}\n"))]);
        let complaint = plan(
            &AccountChange::DeleteKey {
                user: "deploy".into(),
                fingerprint: fingerprint(KEY),
            },
            &reading,
            &|at, _| Ok(disk.get(at).cloned()),
        )
        .expect_err("stale");
        assert!(complaint.contains("no longer holds"), "{complaint}");
    }

    #[test]
    fn every_key_of_an_account_is_taken_by_removing_the_files_the_reading_found_them_in() {
        assert_eq!(
            planned(
                AccountChange::DeleteSshUser {
                    user: "deploy".into()
                },
                &users(),
                &[]
            ),
            vec![Step::Remove {
                path: "/home/deploy/.ssh/authorized_keys".into(),
                holder: Some(1000)
            }]
        );
    }

    #[test]
    fn a_first_key_for_an_account_whose_ssh_directory_points_elsewhere_is_refused_before_anything_is_written()
     {
        let uid = rustix::process::getuid().as_raw();
        if uid == 0 {
            return;
        }
        let gid = rustix::process::getgid().as_raw();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos())
            .unwrap_or(0);
        let home = std::env::temp_dir().join(format!("vigild-plan-home-{stamp}"));
        let elsewhere = std::env::temp_dir().join(format!("vigild-plan-elsewhere-{stamp}"));
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&elsewhere).expect("elsewhere");
        std::fs::write(elsewhere.join("authorized_keys"), "root's key\n").expect("writes");
        std::os::unix::fs::symlink(&elsewhere, home.join(".ssh")).expect("links");
        let mut reading = users();
        reading.items.insert(
            "account|bob".into(),
            json!({"name": "bob", "uid": uid, "gid": gid, "home": home.to_string_lossy()}),
        );

        let complaint = plan(
            &AccountChange::CreateSshUser {
                user: "bob".into(),
                line: OTHER.into(),
            },
            &reading,
            &crate::accounts::files::read,
        )
        .expect_err("a link bob made is not followed");

        assert!(
            complaint.contains("symbolic link"),
            "run as uid {uid}; as root the link would be root's own and followed, so the test \
             says nothing there: {complaint}"
        );
        assert_eq!(
            std::fs::read_to_string(elsewhere.join("authorized_keys")).expect("reads"),
            "root's key\n"
        );
    }

    #[test]
    fn a_logind_session_is_ended_by_loginctl_and_one_only_utmp_saw_by_a_signal_to_its_leader() {
        assert_eq!(
            arguments(&planned(
                AccountChange::DeleteSession {
                    key: "session|deploy|pts/0".into()
                },
                &users(),
                &[]
            )),
            vec![(
                Utility::Loginctl,
                ["terminate-session", "83"].map(String::from).to_vec()
            )]
        );

        let mut reading = users();
        reading.items.insert(
            "session|alice|pts/7".into(),
            json!({"user": "alice", "uid": 1002, "pid": 5150, "session_id": ""}),
        );
        assert_eq!(
            planned(
                AccountChange::DeleteSession {
                    key: "session|alice|pts/7".into()
                },
                &reading,
                &[]
            ),
            vec![Step::Signal {
                pid: 5150,
                uid: Some(1002)
            }]
        );
    }
}
