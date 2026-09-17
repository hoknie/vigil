use rustix::fs::{AtFlags, unlinkat};
use rustix::io::Errno;

use super::reach::place;
use super::regular::existing;

pub fn remove(path: &str, holder: Option<u32>) -> Result<String, String> {
    let place = place(path, holder)?;
    if existing(&place, path)?.is_none() {
        return Ok(format!("{path} was already gone"));
    }
    match unlinkat(&place.parent, place.name.as_str(), AtFlags::empty()) {
        Ok(()) => Ok(format!("{path} removed")),
        Err(Errno::NOENT) => Ok(format!("{path} was already gone")),
        Err(error) => Err(format!("{path} could not be removed: {error}")),
    }
}
