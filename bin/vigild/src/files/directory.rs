use rustix::fs::{
    AtFlags, FileType, Gid, Mode, OFlags, Uid, fchmod, fchown, mkdirat, openat, statat,
};
use rustix::io::Errno;

use super::mode::bits;
use super::reach::place;
use super::regular::linked;

pub fn directory(
    path: &str,
    uid: u32,
    gid: u32,
    mode: u32,
    holder: Option<u32>,
) -> Result<String, String> {
    let place = place(path, holder)?;
    match statat(
        &place.parent,
        place.name.as_str(),
        AtFlags::SYMLINK_NOFOLLOW,
    ) {
        Ok(stat) => {
            return match FileType::from_raw_mode(stat.st_mode) {
                FileType::Directory => Ok(format!("{path} was there")),
                FileType::Symlink => Err(linked(path)),
                _ => Err(format!("{path} is there and is not a directory")),
            };
        }
        Err(Errno::NOENT) => {}
        Err(error) => return Err(format!("{path} could not be looked at: {error}")),
    }

    mkdirat(
        &place.parent,
        place.name.as_str(),
        Mode::from_bits_truncate(0o700),
    )
    .map_err(|error| format!("{path} could not be created: {error}"))?;
    let made = openat(
        &place.parent,
        place.name.as_str(),
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| format!("{path} was created and could not be opened: {error}"))?;
    fchown(
        &made,
        Some(Uid::from_raw_unchecked(uid)),
        Some(Gid::from_raw_unchecked(gid)),
    )
    .and_then(|()| fchmod(&made, bits(mode)))
    .map_err(|error| format!("{path} was created but not handed to its account: {error}"))?;
    Ok(format!("{path} created"))
}
