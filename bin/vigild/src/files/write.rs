use rustix::fs::Mode;

use super::candidate::{candidate, fill, install};
use super::mode::bits;
use super::reach::place;
use super::regular::existing;

pub fn write(
    path: &str,
    text: &str,
    uid: u32,
    gid: u32,
    mode: u32,
    holder: Option<u32>,
) -> Result<String, String> {
    let place = place(path, holder)?;
    let (uid, gid, mode) = match existing(&place, path)? {
        Some(stat) => (stat.st_uid, stat.st_gid, Mode::from_raw_mode(stat.st_mode)),
        None => (uid, gid, bits(mode)),
    };

    let candidate = candidate(&place.name);
    fill(&place, &candidate, text, uid, gid, mode)
        .map_err(|error| format!("{path} could not be written: {error}"))?;
    install(&place, &candidate, path)?;
    Ok(format!("{path} written"))
}
