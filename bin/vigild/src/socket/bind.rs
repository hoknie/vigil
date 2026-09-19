use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

pub fn bind(path: &Path) -> Result<UnixListener, String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty() && !parent.exists())
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("{} cannot be created: {error}", parent.display()))?;
        let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
    }

    remove_stale_socket(path)?;

    if directory_is_private(path) {
        let listener = UnixListener::bind(path).map_err(|error| {
            format!(
                "the console socket {} cannot be created: {error}",
                path.display()
            )
        })?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("{} cannot be made private: {error}", path.display()))?;
        return Ok(listener);
    }

    let temporary = temporary_path(path);
    let _ = fs::remove_file(&temporary);
    let listener = UnixListener::bind(&temporary).map_err(|error| {
        format!(
            "the console socket {} cannot be created: {error}",
            path.display()
        )
    })?;
    fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("{} cannot be made private: {error}", path.display()))?;
    fs::rename(&temporary, path).map_err(|error| {
        format!(
            "the console socket cannot be put at {}: {error}",
            path.display()
        )
    })?;

    Ok(listener)
}

fn directory_is_private(path: &Path) -> bool {
    let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) else {
        return false;
    };
    let Ok(metadata) = fs::metadata(parent) else {
        return false;
    };

    use std::os::unix::fs::MetadataExt;
    metadata.uid() == current_uid() && metadata.mode() & 0o077 == 0
}

fn current_uid() -> u32 {
    rustix::process::getuid().as_raw()
}

fn remove_stale_socket(path: &Path) -> Result<(), String> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };

    if metadata.file_type().is_socket() {
        return fs::remove_file(path).map_err(|error| {
            format!(
                "the previous socket {} is in the way: {error}",
                path.display()
            )
        });
    }

    Err(format!(
        "{} already exists and is not a socket: refusing to remove it",
        path.display()
    ))
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.new", std::process::id()));
    path.with_file_name(name)
}

#[cfg(test)]
mod tests_directory {
    use std::os::unix::fs::MetadataExt;

    use super::*;

    fn directory(name: &str, mode: u32) -> PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "vigil-listener-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos()
                    + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
                .unwrap_or(0)
        ));
        fs::create_dir_all(&path).expect("temp dir");
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("chmod");
        path
    }

    #[test]
    fn a_directory_only_we_can_enter_is_private_and_one_others_can_is_not() {
        let ours = directory("ours", 0o700);
        let shared = directory("shared", 0o755);

        assert!(directory_is_private(&ours.join("vigil.sock")));
        assert!(
            !directory_is_private(&shared.join("vigil.sock")),
            "a socket path under a directory anybody may enter has to take the careful route"
        );

        let _ = fs::remove_dir_all(&ours);
        let _ = fs::remove_dir_all(&shared);
    }

    #[test]
    fn a_directory_that_cannot_be_read_is_answered_no() {
        assert!(!directory_is_private(Path::new(
            "/vigil-does-not-exist/vigil.sock"
        )));
        assert!(!directory_is_private(Path::new("vigil.sock")));
    }

    #[test]
    fn our_own_uid_is_the_one_the_kernel_says_on_every_system() {
        let ours = directory("uid", 0o700);

        assert_eq!(
            fs::metadata(&ours).map(|made| made.uid()).ok(),
            Some(current_uid()),
            "a uid read wrongly makes every directory look like somebody else's, and a Mac \
             has no /proc/self/status to read it from"
        );
        let _ = fs::remove_dir_all(&ours);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vigil-socket-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn the_socket_is_readable_by_nobody_but_its_owner() {
        let path = scratch("mode").join("vigil.sock");

        let _listener = bind(&path).expect("binds");

        let mode = fs::metadata(&path).expect("exists").permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "a terminal into a privileged agent is not public"
        );
        assert!(
            fs::symlink_metadata(&path)
                .expect("exists")
                .file_type()
                .is_socket()
        );
        let _ = fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn a_socket_left_by_a_previous_run_is_replaced_rather_than_refused() {
        let path = scratch("stale").join("vigil.sock");
        let first = bind(&path).expect("binds");
        drop(first);

        let _second = bind(&path).expect("a restart must not need the file removed by hand");

        let _ = fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn a_directory_that_is_not_there_is_made_and_made_private_because_boot_empties_it() {
        let dir = scratch("made");
        let path = dir.join("run").join("vigil").join("vigil.sock");

        let _listener = bind(&path).expect("binds");

        let parent = path.parent().expect("parent");
        let mode = fs::metadata(parent).expect("made").permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o700,
            "/run on Linux and /var/run on macOS are emptied at boot, and the daemon makes the \
             directory of its socket again, closed to everybody else"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_directory_that_was_already_there_is_left_as_it_was_found() {
        let dir = scratch("keep");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("chmod");

        let _listener = bind(&dir.join("vigil.sock")).expect("binds");

        let mode = fs::metadata(&dir).expect("exists").permissions().mode() & 0o777;
        assert_eq!(mode, 0o755, "the directory was not ours to change");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn something_at_that_path_that_is_not_a_socket_is_never_deleted() {
        let dir = scratch("not-a-socket");
        let path = dir.join("important.conf");
        fs::write(&path, "keep me\n").expect("write");

        let error = bind(&path).expect_err("must refuse");

        assert!(error.contains("not a socket"), "{error}");
        assert_eq!(fs::read_to_string(&path).expect("still there"), "keep me\n");
        let _ = fs::remove_dir_all(&dir);
    }
}
