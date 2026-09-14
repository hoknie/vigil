use rustix::fs::{AtFlags, unlinkat};

use super::candidate::{candidate, fill, install};
use super::mode::bits;
use super::reach::place;
use super::regular::existing;

pub fn write_checked(
    path: &str,
    text: &str,
    mode: u32,
    check: &dyn Fn(&str) -> Result<String, String>,
) -> Result<String, String> {
    let place = place(path, None)?;
    let (uid, gid) = match existing(&place, path)? {
        Some(stat) => (stat.st_uid, stat.st_gid),
        None => (0, 0),
    };

    let candidate = candidate(&place.name);
    fill(&place, &candidate, text, uid, gid, bits(mode))
        .map_err(|error| format!("{path} could not be written: {error}"))?;

    let beside = match path.rsplit_once('/') {
        Some((directory, _)) => format!("{directory}/{candidate}"),
        None => candidate.clone(),
    };
    if let Err(refused) = check(&beside) {
        let _ = unlinkat(&place.parent, candidate.as_str(), AtFlags::empty());
        return Err(refused);
    }

    install(&place, &candidate, path)?;
    Ok(format!("{path} put in place"))
}
