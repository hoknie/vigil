use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use vigil_collect::parse_passwd_entries;
use vigil_model::Snapshot;

use super::keys::read_authorized_keys;
use super::sessions::read_sessions;
use super::{GROUP, PASSWD, SHADOW, SUDOERS, SUDOERS_DIRECTORY};
use crate::parsers::{
    AccountsReading, ShadowFacts, SudoGrant, accounts_snapshot, parse_group, parse_shadow,
    parse_sudoers,
};
use vigil_collect::CollectError;

pub fn reading(taken_at: &str) -> Result<Snapshot, CollectError> {
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
    let seen = read_sessions(&passwd);

    Ok(accounts_snapshot(
        taken_at,
        &AccountsReading {
            passwd: &passwd,
            groups: &groups,
            shadow: shadow.as_ref(),
            sudo: &sudo,
            keys: &keys,
            sessions: &seen.sessions,
            session_sources: &seen.sources,
        },
    ))
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
