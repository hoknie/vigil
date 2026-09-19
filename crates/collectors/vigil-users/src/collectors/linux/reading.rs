use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;

use vigil_collect::parse_passwd_entries;
use vigil_model::Snapshot;

use super::keys::read_authorized_keys;
use super::sessions::read_sessions;
use super::sudoers::read_sudoers;
use super::{GROUP, PASSWD, SHADOW};
use crate::parsers::{AccountsReading, ShadowFacts, accounts_snapshot, parse_group, parse_shadow};
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
