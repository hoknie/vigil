use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::OwnedFd;

use rustix::fs::{
    AtFlags, CWD, FileType, Gid, Mode, OFlags, Stat, Uid, fchmod, fchown, fstat, mkdirat, openat,
    readlinkat, renameat, statat, unlinkat,
};
use rustix::io::{Errno, fcntl_dupfd_cloexec as duplicate};

const MOST_LINKS_FOLLOWED: usize = 8;

struct Place {
    parent: OwnedFd,
    name: String,
}

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

fn place(path: &str, holder: Option<u32>) -> Result<Place, String> {
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

fn existing(place: &Place, path: &str) -> Result<Option<Stat>, String> {
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

fn plain(stat: &Stat, path: &str) -> Result<(), String> {
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

fn fill(
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

fn install(place: &Place, candidate: &str, path: &str) -> Result<(), String> {
    renameat(&place.parent, candidate, &place.parent, place.name.as_str()).map_err(|error| {
        let _ = unlinkat(&place.parent, candidate, AtFlags::empty());
        format!("{path} could not be put in place: {error}")
    })
}

fn candidate(name: &str) -> String {
    format!(".vigil-{name}.candidate")
}

#[cfg(target_os = "linux")]
fn bits(mode: u32) -> Mode {
    Mode::from_bits_truncate(mode)
}

#[cfg(not(target_os = "linux"))]
fn bits(mode: u32) -> Mode {
    Mode::from_bits_truncate(mode as rustix::fs::RawMode)
}

fn linked(path: &str) -> String {
    format!(
        "{path} goes through a symbolic link that root did not put there: the agent writes as \
         root, and following a link an account put in its own directory is writing wherever \
         that account pointed it"
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
    use std::path::{Path, PathBuf};

    use super::*;

    fn scratch(named: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "vigild-accounts-{named}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&directory).expect("temp dir");
        directory
    }

    fn ours() -> (u32, u32) {
        (
            rustix::process::getuid().as_raw(),
            rustix::process::getgid().as_raw(),
        )
    }

    fn text(path: &Path) -> &str {
        path.to_str().expect("utf-8")
    }

    #[test]
    fn a_file_written_again_keeps_the_owner_and_the_mode_it_had_and_leaves_no_candidate() {
        let directory = scratch("keep");
        let path = directory.join("authorized_keys");
        fs::write(&path, "old\n").expect("writes");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).expect("chmod");
        let (uid, gid) = ours();

        write(text(&path), "new\n", uid, gid, 0o600, Some(uid)).expect("written");

        assert_eq!(fs::read_to_string(&path).expect("reads"), "new\n");
        assert_eq!(
            fs::metadata(&path).expect("there").mode() & 0o777,
            0o640,
            "the mode a person gave the file is theirs; the one passed in is for a new file"
        );
        assert!(!directory.join(candidate("authorized_keys")).exists());
        assert_eq!(read(text(&path), Some(uid)), Ok(Some("new\n".to_string())));
    }

    #[test]
    fn a_new_file_is_given_the_mode_it_was_asked_for() {
        let directory = scratch("new");
        let path = directory.join("authorized_keys");
        let (uid, gid) = ours();

        write(text(&path), "key\n", uid, gid, 0o600, Some(uid)).expect("written");

        assert_eq!(fs::metadata(&path).expect("there").mode() & 0o777, 0o600);
    }

    #[test]
    fn a_link_where_a_key_file_should_be_is_refused_rather_than_followed() {
        let directory = scratch("link");
        let target = directory.join("elsewhere");
        fs::write(&target, "untouched\n").expect("writes");
        let path = directory.join("authorized_keys");
        symlink(&target, &path).expect("links");
        let (uid, gid) = ours();

        let complaint = write(text(&path), "key\n", uid, gid, 0o600, Some(uid))
            .expect_err("a link is not written through");

        assert!(complaint.contains("symbolic link"), "{complaint}");
        assert_eq!(fs::read_to_string(&target).expect("reads"), "untouched\n");
        assert!(read(text(&path), Some(uid)).is_err());
        assert!(remove(text(&path), Some(uid)).is_err());
        assert!(target.exists());
    }

    #[test]
    fn an_ssh_directory_an_account_pointed_at_another_directory_is_neither_read_written_removed_nor_made()
     {
        let home = scratch("pointed");
        let elsewhere = scratch("roots-keys");
        let keys = elsewhere.join("authorized_keys");
        fs::write(&keys, "root's key\n").expect("writes");
        symlink(&elsewhere, home.join(".ssh")).expect("links");
        let (uid, gid) = ours();
        let through = home.join(".ssh").join("authorized_keys");

        if uid == 0 {
            return;
        }
        let refusals = [
            read(text(&through), Some(uid)).map(|_| String::new()),
            write(text(&through), "bob's key\n", uid, gid, 0o600, Some(uid)),
            remove(text(&through), Some(uid)),
            directory(
                text(&home.join(".ssh").join("inner")),
                uid,
                gid,
                0o700,
                Some(uid),
            ),
        ];

        for refused in refusals {
            let complaint = refused.expect_err("a link an account made is not followed");
            assert!(complaint.contains("symbolic link"), "{complaint}");
        }
        assert_eq!(
            fs::read_to_string(&keys).expect("still there"),
            "root's key\n",
            "the file at the other end of the link is the one an account wanted root to rewrite"
        );
        assert!(!elsewhere.join("inner").exists());
        assert!(!elsewhere.join(candidate("authorized_keys")).exists());
    }

    #[test]
    fn a_link_root_put_in_the_path_is_followed_because_where_homes_live_is_roots_decision() {
        let real = scratch("real-home");
        fs::write(real.join("authorized_keys"), "key\n").expect("writes");
        let parent = scratch("links");
        let link = parent.join("home");
        symlink(&real, &link).expect("links");
        let (uid, _) = ours();

        let through = read(text(&link.join("authorized_keys")), Some(uid));

        match uid {
            0 => assert_eq!(
                through,
                Ok(Some("key\n".to_string())),
                "a link owned by root is followed, as /home to /var/home is on some hosts"
            ),
            _ => assert!(
                through.expect_err("not root").contains("symbolic link"),
                "run as uid {uid}, this link is not root's and is refused; the test proves \
                 the following only when run as root, as it is in the Linux container"
            ),
        }
    }

    #[test]
    fn a_key_file_with_a_second_hard_link_is_neither_read_nor_rewritten() {
        let directory = scratch("hard");
        let original = directory.join("somebody-elses");
        fs::write(&original, "theirs\n").expect("writes");
        let path = directory.join("authorized_keys");
        fs::hard_link(&original, &path).expect("links");
        let (uid, gid) = ours();

        let complaint = read(text(&path), Some(uid)).expect_err("two names");
        assert!(complaint.contains("hard link"), "{complaint}");
        assert!(write(text(&path), "mine\n", uid, gid, 0o600, Some(uid)).is_err());
        assert_eq!(fs::read_to_string(&original).expect("reads"), "theirs\n");
    }

    #[test]
    fn a_key_file_in_a_directory_kept_by_somebody_other_than_its_account_or_root_is_refused() {
        let directory = scratch("kept");
        let (uid, gid) = ours();
        let stranger = uid.wrapping_add(4242).max(1);

        let complaint = write(
            text(&directory.join("authorized_keys")),
            "key\n",
            uid,
            gid,
            0o600,
            Some(stranger),
        );

        match uid {
            0 => assert!(
                complaint.is_ok(),
                "a directory root keeps is allowed for any account"
            ),
            _ => assert!(
                complaint.expect_err("kept by another").contains("neither"),
                "the directory belongs to uid {uid}, not to uid {stranger} nor root"
            ),
        }
    }

    #[test]
    fn a_directory_that_is_there_is_left_as_it_is_and_one_that_is_not_is_made_with_its_mode() {
        let parent = scratch("directory");
        let path = parent.join(".ssh");
        let (uid, gid) = ours();

        assert!(
            directory(text(&path), uid, gid, 0o700, Some(uid))
                .expect("made")
                .contains("created")
        );
        assert_eq!(fs::metadata(&path).expect("there").mode() & 0o777, 0o700);
        assert!(
            directory(text(&path), uid, gid, 0o700, Some(uid))
                .expect("there")
                .contains("was there")
        );
    }

    #[test]
    fn a_file_that_is_already_gone_is_not_a_failure_to_remove_it() {
        let directory = scratch("gone");
        let path = directory.join("authorized_keys");
        let (uid, _) = ours();

        assert!(
            remove(text(&path), Some(uid))
                .expect("nothing to remove")
                .contains("already gone")
        );
        assert_eq!(read(text(&path), Some(uid)), Ok(None));
    }

    #[test]
    fn a_checked_file_that_fails_its_check_leaves_the_old_one_and_no_candidate_behind() {
        let directory = scratch("checked");
        let path = directory.join("deploy");
        fs::write(&path, "deploy ALL=(ALL) ALL\n").expect("writes");

        let refused = write_checked(text(&path), "garbage\n", 0o440, &|candidate| {
            assert!(
                Path::new(candidate).exists(),
                "{candidate} is checked where it lies"
            );
            Err("visudo said no".to_string())
        });

        assert_eq!(refused, Err("visudo said no".to_string()));
        assert_eq!(
            fs::read_to_string(&path).expect("reads"),
            "deploy ALL=(ALL) ALL\n"
        );
        assert!(!directory.join(candidate("deploy")).exists());
    }

    #[test]
    fn a_candidate_sits_beside_its_file_under_a_name_sudo_skips_in_its_include_directory() {
        assert_eq!(
            candidate("deploy"),
            ".vigil-deploy.candidate",
            "sudo reads no file in /etc/sudoers.d whose name holds a dot, so a candidate left \
             behind by a crash is never a live grant"
        );
    }
}
