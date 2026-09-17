use std::os::fd::OwnedFd;

use rustix::fs::{AtFlags, CWD, FileType, Mode, OFlags, fstat, openat, readlinkat, statat};
use rustix::io::{Errno, fcntl_dupfd_cloexec as duplicate};

use super::place::Place;
use super::regular::linked;

const MOST_LINKS_FOLLOWED: usize = 8;

pub fn place(path: &str, holder: Option<u32>) -> Result<Place, String> {
    let Some(relative) = path.strip_prefix('/') else {
        return Err(format!("{path} is not an absolute path"));
    };
    let mut parts: Vec<&str> = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let Some(name) = parts.pop() else {
        return Err(format!("{path} names no file"));
    };
    if name == "." || name == ".." {
        return Err(format!("{path} does not end in the name of a file"));
    }

    let parent = descend(root(path)?, &parts, path, false, 0)?;

    if let Some(holder) = holder {
        let owner = fstat(&parent)
            .map_err(|error| format!("the directory of {path} could not be looked at: {error}"))?
            .st_uid;
        if owner != holder && owner != 0 {
            return Err(format!(
                "the directory holding {path} belongs to uid {owner}, which is neither the \
                 account the file is for nor root: the agent does not rewrite a key file that \
                 somebody else keeps"
            ));
        }
    }

    Ok(Place {
        parent,
        name: name.to_string(),
    })
}

fn descend(
    mut at: OwnedFd,
    parts: &[&str],
    path: &str,
    inside_a_link: bool,
    depth: usize,
) -> Result<OwnedFd, String> {
    for part in parts {
        if *part == "." {
            continue;
        }
        if *part == ".." && !inside_a_link {
            return Err(format!(
                "{path} climbs with .., which is not a path the reading writes"
            ));
        }
        at = step(&at, part, path, depth)?;
    }
    Ok(at)
}

fn step(at: &OwnedFd, part: &str, path: &str, depth: usize) -> Result<OwnedFd, String> {
    match openat(
        at,
        part,
        directory_flags() | OFlags::NOFOLLOW,
        Mode::empty(),
    ) {
        Ok(opened) => return Ok(opened),
        Err(Errno::LOOP) | Err(Errno::NOTDIR) => {}
        Err(error) => return Err(format!("{part} in {path} could not be opened: {error}")),
    }

    let link = statat(at, part, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| format!("{part} in {path} could not be looked at: {error}"))?;
    if FileType::from_raw_mode(link.st_mode) != FileType::Symlink {
        return Err(format!("{part} in {path} is not a directory"));
    }
    if link.st_uid != 0 {
        return Err(linked(path));
    }
    if depth >= MOST_LINKS_FOLLOWED {
        return Err(format!(
            "{path} goes through more than {MOST_LINKS_FOLLOWED} links"
        ));
    }

    let target = readlinkat(at, part, Vec::new())
        .map_err(|error| format!("{part} in {path} could not be read as a link: {error}"))?;
    let again = statat(at, part, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| format!("{part} in {path} could not be looked at: {error}"))?;
    if again.st_ino != link.st_ino || again.st_uid != 0 {
        return Err(format!(
            "{part} in {path} changed while the agent was following it"
        ));
    }

    let target = target.to_string_lossy().into_owned();
    let start = match target.starts_with('/') {
        true => root(path)?,
        false => {
            duplicate(at, 0).map_err(|error| format!("{path} could not be followed: {error}"))?
        }
    };
    let parts: Vec<&str> = target.split('/').filter(|part| !part.is_empty()).collect();
    descend(start, &parts, path, true, depth + 1)
}

fn root(path: &str) -> Result<OwnedFd, String> {
    openat(CWD, "/", directory_flags(), Mode::empty())
        .map_err(|error| format!("/ could not be opened to reach {path}: {error}"))
}

#[cfg(target_os = "linux")]
fn directory_flags() -> OFlags {
    OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC
}

#[cfg(not(target_os = "linux"))]
fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC
}
