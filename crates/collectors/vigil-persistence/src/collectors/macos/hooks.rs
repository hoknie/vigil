use std::path::Path;

use super::files::{Unread, read_capped};
use super::places::{HOOKS, LOGIN_HOOKS};
use super::scripts::described;
use crate::parsers::{PlistValue, ScriptFamily, WatchedScript, parse_plist};

pub(super) fn read_hooks() -> Result<Vec<WatchedScript>, Unread> {
    let bytes = match read_capped(Path::new(LOGIN_HOOKS)) {
        Ok(bytes) => bytes,
        Err(error) if vigil_collect::absent(&error) => return Ok(Vec::new()),
        Err(error) => return Err(Unread::of(format!("{LOGIN_HOOKS}: {error}"), &error)),
    };
    let preferences = parse_plist(&bytes)
        .map_err(|refusal| Unread::Failed(format!("{LOGIN_HOOKS}: {refusal}")))?;

    Ok(HOOKS
        .iter()
        .filter_map(|hook| preferences.get(hook).and_then(PlistValue::as_str))
        .filter(|path| path.starts_with('/'))
        .map(|path| described(Path::new(path), ScriptFamily::Boot))
        .collect())
}
