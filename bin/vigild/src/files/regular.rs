use rustix::fs::{AtFlags, FileType, Stat, statat};
use rustix::io::Errno;

use super::place::Place;

pub fn existing(place: &Place, path: &str) -> Result<Option<Stat>, String> {
    match statat(
        &place.parent,
        place.name.as_str(),
        AtFlags::SYMLINK_NOFOLLOW,
    ) {
        Ok(stat) => plain(&stat, path).map(|()| Some(stat)),
        Err(Errno::NOENT) => Ok(None),
        Err(error) => Err(format!("{path} could not be looked at: {error}")),
    }
}

pub fn plain(stat: &Stat, path: &str) -> Result<(), String> {
    match FileType::from_raw_mode(stat.st_mode) {
        FileType::RegularFile if stat.st_nlink > 1 => Err(format!(
            "{path} has {} names on this disk: a hard link put in an account's directory is \
             somebody else's file, and the agent does not read or rewrite it as root",
            stat.st_nlink
        )),
        FileType::RegularFile => Ok(()),
        FileType::Symlink => Err(linked(path)),
        _ => Err(format!("{path} is there and is not a regular file")),
    }
}

pub fn linked(path: &str) -> String {
    format!(
        "{path} goes through a symbolic link that root did not put there: the agent writes as \
         root, and following a link an account put in its own directory is writing wherever \
         that account pointed it"
    )
}
