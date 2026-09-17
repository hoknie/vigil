use std::fs::File;
use std::io::Write;

use rustix::fs::{AtFlags, Gid, Mode, OFlags, Uid, fchmod, fchown, openat, renameat, unlinkat};
use rustix::io::Errno;

use super::place::Place;

pub fn fill(
    place: &Place,
    candidate: &str,
    text: &str,
    uid: u32,
    gid: u32,
    mode: Mode,
) -> Result<(), Errno> {
    let _ = unlinkat(&place.parent, candidate, AtFlags::empty());
    let opened = openat(
        &place.parent,
        candidate,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::from_bits_truncate(0o600),
    )?;
    let mut file = File::from(opened);

    let filled = file
        .write_all(text.as_bytes())
        .map_err(|error| Errno::from_io_error(&error).unwrap_or(Errno::IO))
        .and_then(|()| {
            fchown(
                &file,
                Some(Uid::from_raw_unchecked(uid)),
                Some(Gid::from_raw_unchecked(gid)),
            )
        })
        .and_then(|()| fchmod(&file, mode))
        .and_then(|()| {
            file.sync_all()
                .map_err(|error| Errno::from_io_error(&error).unwrap_or(Errno::IO))
        });
    if filled.is_err() {
        let _ = unlinkat(&place.parent, candidate, AtFlags::empty());
    }
    filled
}

pub fn install(place: &Place, candidate: &str, path: &str) -> Result<(), String> {
    renameat(&place.parent, candidate, &place.parent, place.name.as_str()).map_err(|error| {
        let _ = unlinkat(&place.parent, candidate, AtFlags::empty());
        format!("{path} could not be put in place: {error}")
    })
}

pub fn candidate(name: &str) -> String {
    format!(".vigil-{name}.candidate")
}
