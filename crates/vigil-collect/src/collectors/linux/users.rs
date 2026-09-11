use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use vigil_model::{Rfc3339, Snapshot};

use crate::parsers::{
    AccountsReading, PasswdEntry, Session, ShadowFacts, SudoGrant, UserKeyFile, accounts_snapshot,
    parse_authorized_keys, parse_group, parse_passwd_entries, parse_shadow, parse_sudoers,
    parse_utmp,
};
use crate::{CollectError, Collector, Health};

const PASSWD: &str = "/etc/passwd";
const GROUP: &str = "/etc/group";
const SHADOW: &str = "/etc/shadow";
const SUDOERS: &str = "/etc/sudoers";
const SUDOERS_DIRECTORY: &str = "/etc/sudoers.d";

const UTMP: &[&str] = &["/var/run/utmp", "/run/utmp"];

const KEY_FILES: &[&str] = &["authorized_keys", "authorized_keys2"];

pub struct UsersCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl UsersCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        UsersCollector { now: Box::new(now) }
    }
}

impl Collector for UsersCollector {
    fn name(&self) -> &'static str {
        "users"
    }

    fn available(&self) -> Health {
        if fs::read_to_string(PASSWD).is_err() {
            return Health::Unavailable(format!("{PASSWD} cannot be read"));
        }

        let mut missing: Vec<String> = Vec::new();

        if let Err(error) = fs::read_to_string(SHADOW) {
            missing.push(match error.kind() {
                ErrorKind::PermissionDenied => format!(
                    "{SHADOW} is not readable: lock state and password dates will be missing; run as root"
                ),
                _ => format!("{SHADOW} is not present: lock state and password dates will be missing"),
            });
        }
        match fs::read_to_string(SUDOERS) {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) if error.kind() == ErrorKind::PermissionDenied => missing.push(format!(
                "{SUDOERS} is not readable: grants made there will not be seen; run as root"
            )),
            Err(error) => missing.push(format!(
                "{SUDOERS} could not be read ({error}): grants made there will not be seen"
            )),
        }
        if utmp_path().is_none() {
            missing.push(
                "logins are not recorded in a form this build reads: active sessions will be missing"
                    .into(),
            );
        }

        match missing.is_empty() {
            true => Health::Ok,
            false => Health::Degraded(missing.join("; ")),
        }
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let passwd = match fs::read_to_string(PASSWD) {
            Ok(text) => parse_passwd_entries(&text),
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                return Err(CollectError::Denied(PASSWD.into()));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(CollectError::Absent(PASSWD.into()));
            }
            Err(error) => return Err(CollectError::Unreadable(format!("{PASSWD}: {error}"))),
        };
        if passwd.is_empty() {
            return Err(CollectError::Unreadable(format!(
                "{PASSWD} held no line this build understands"
            )));
        }

        let groups = fs::read_to_string(GROUP)
            .map(|text| parse_group(&text))
            .unwrap_or_default();
        let shadow = read_shadow();
        let sudo = read_sudoers();
        let keys = read_authorized_keys(&passwd);
        let sessions = read_sessions();

        Ok(accounts_snapshot(
            &(self.now)(),
            &AccountsReading {
                passwd: &passwd,
                groups: &groups,
                shadow: shadow.as_ref(),
                sudo: &sudo,
                keys: &keys,
                sessions: sessions.as_deref(),
            },
        ))
    }
}

fn read_shadow() -> Option<BTreeMap<String, ShadowFacts>> {
    fs::read_to_string(SHADOW)
        .ok()
        .map(|text| parse_shadow(&text))
}

fn read_sudoers() -> Vec<SudoGrant> {
    let mut grants = Vec::new();

    if let Ok(text) = fs::read_to_string(SUDOERS) {
        grants.extend(parse_sudoers(&text, SUDOERS).grants);
    }

    let mut files: Vec<PathBuf> = fs::read_dir(SUDOERS_DIRECTORY)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();

    for path in files {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.ends_with('~') || name.contains('.') {
            continue;
        }
        if let Ok(text) = fs::read_to_string(&path) {
            grants.extend(parse_sudoers(&text, &path.to_string_lossy()).grants);
        }
    }

    grants
}

fn read_authorized_keys(passwd: &[PasswdEntry]) -> Vec<UserKeyFile> {
    let mut files = Vec::new();

    for entry in passwd {
        if entry.home.is_empty() {
            continue;
        }
        let directory = Path::new(&entry.home).join(".ssh");

        match fs::metadata(&directory) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => continue,
            Err(error)
                if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
            {
                continue;
            }
            Err(_) => {
                files.push(unreadable(entry, &directory));
                continue;
            }
        }

        let mut refused = false;
        for file_name in KEY_FILES {
            let path = directory.join(file_name);
            match fs::read_to_string(&path) {
                Ok(text) => files.push(UserKeyFile {
                    user: entry.name.clone(),
                    uid: entry.uid,
                    path: path.to_string_lossy().into_owned(),
                    readable: true,
                    keys: parse_authorized_keys(&text),
                }),
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(_) => refused = true,
            }
        }
        if refused {
            files.push(unreadable(entry, &directory));
        }
    }

    files
}

fn unreadable(entry: &PasswdEntry, directory: &Path) -> UserKeyFile {
    UserKeyFile {
        user: entry.name.clone(),
        uid: entry.uid,
        path: directory.to_string_lossy().into_owned(),
        readable: false,
        keys: Vec::new(),
    }
}

fn utmp_path() -> Option<&'static str> {
    UTMP.iter()
        .copied()
        .find(|path| fs::metadata(path).is_ok_and(|meta| meta.len() > 0))
}

fn read_sessions() -> Option<Vec<Session>> {
    let path = utmp_path()?;
    parse_utmp(&fs::read(path).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_this_host_and_names_itself_in_the_snapshot() {
        let collector = UsersCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

        let snapshot = collector
            .collect()
            .expect("/etc/passwd is readable on Linux");

        assert_eq!(snapshot.source, "users");
        assert_eq!(snapshot.taken_at, "2026-09-09T12:00:00.000Z");
        assert!(
            snapshot.items.keys().any(|key| key.starts_with("account|")),
            "every host has accounts"
        );
        for (key, item) in &snapshot.items {
            assert!(key.contains('|'), "key shape: {key}");
            if key.starts_with("account|") {
                assert!(
                    item.get("shadow_readable").is_some(),
                    "{key} lost the flag that separates 'could not read' from 'not locked'"
                );
            }
        }
    }

    #[test]
    fn nothing_that_could_be_cracked_reaches_the_snapshot_of_this_host() {
        let collector = UsersCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());
        let snapshot = collector.collect().expect("readable");

        let printed = serde_json::to_string(&snapshot).expect("serialises");
        for marker in ["$1$", "$5$", "$6$", "$y$", "$2b$"] {
            assert!(
                !printed.contains(marker),
                "a password hash marker reached the snapshot: {marker}"
            );
        }
    }
}
