use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::candidate::candidate;
use super::{directory, read, remove, write, write_checked};

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
