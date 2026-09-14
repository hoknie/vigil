use serde_json::Value;
use vigil_model::Snapshot;

use super::keys;
use super::reader::{Reader, present};
use crate::accounts::fields::{flag, item, key_rows, number, sources, text};
use crate::accounts::step::Step;

const KEY_FILE_MODE: u32 = 0o600;

const KEY_DIRECTORY_MODE: u32 = 0o700;

pub fn written(path: String, text: String, uid: u32, gid: u32) -> Step {
    Step::Write {
        path,
        text,
        uid,
        gid,
        mode: KEY_FILE_MODE,
    }
}

pub fn added(
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

pub fn owner<'a>(reading: &'a Snapshot, user: &str) -> Result<(u32, u32, &'a str), String> {
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

pub fn file_of_key(
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

pub fn no_longer(path: &str, fingerprint: &str) -> String {
    format!("{path} no longer holds the key {fingerprint}: the reading is older than the file")
}

fn id(account: &Value, field: &str) -> Result<u32, String> {
    number(account, field)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("the reading carries no {field} for this account"))
}
