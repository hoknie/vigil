use std::path::Path;

use super::files::read_capped;
use super::places::{HOOKS, LOGIN_HOOKS};
use super::scripts::described;
use crate::parsers::{PlistValue, ScriptFamily, WatchedScript, parse_plist};

pub(super) fn read_hooks() -> Result<Vec<WatchedScript>, String> {
    let bytes = match read_capped(Path::new(LOGIN_HOOKS)) {
        Ok(bytes) => bytes,
        Err(error) if vigil_collect::absent(&error) => return Ok(Vec::new()),
        Err(error) => return Err(format!("{LOGIN_HOOKS}: {error}")),
    };
    let preferences = parse_plist(&bytes).map_err(|refusal| format!("{LOGIN_HOOKS}: {refusal}"))?;

    Ok(HOOKS
        .iter()
        .filter_map(|hook| preferences.get(hook).and_then(PlistValue::as_str))
        .filter(|path| path.starts_with('/'))
        .map(|path| described(Path::new(path), ScriptFamily::Boot))
        .collect())
}
