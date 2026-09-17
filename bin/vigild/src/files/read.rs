use std::fs::File;
use std::io::Read;

use rustix::fs::{Mode, OFlags, fstat, openat};

use super::reach::place;
use super::regular::{existing, plain};

pub fn read(path: &str, holder: Option<u32>) -> Result<Option<String>, String> {
    let place = place(path, holder)?;
    if existing(&place, path)?.is_none() {
        return Ok(None);
    }

    let opened = openat(
        &place.parent,
        place.name.as_str(),
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| format!("{path} could not be opened: {error}"))?;
    plain(
        &fstat(&opened).map_err(|error| format!("{path} could not be looked at: {error}"))?,
        path,
    )?;

    let mut text = String::new();
    File::from(opened)
        .read_to_string(&mut text)
        .map_err(|error| format!("{path} could not be read: {error}"))?;
    Ok(Some(text))
}
