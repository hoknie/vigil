use std::fs;
use std::io::{BufReader, Write};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use vigil_model::{ProtocolError, Response, Rfc3339};

use super::{Shared, serve};

const MAX_SESSIONS: usize = 8;

const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

pub fn listen(path: &str, shared: Shared, now: fn() -> Rfc3339) -> Result<(), String> {
    let listener = bind(Path::new(path))?;

    let sessions = Arc::new(AtomicUsize::new(0));
    let path_for_thread = path.to_string();

    thread::Builder::new()
        .name("vigil-socket".to_string())
        .spawn(move || accept_loop(listener, shared, now, sessions, path_for_thread))
        .map_err(|error| format!("the socket thread could not be started: {error}"))?;

    Ok(())
}

fn bind(path: &Path) -> Result<UnixListener, String> {
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
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix("Uid:"))
                .and_then(|value| value.split_whitespace().next().map(str::to_string))
        })
        .and_then(|uid| uid.parse().ok())
        .unwrap_or(u32::MAX)
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

fn accept_loop(
    listener: UnixListener,
    shared: Shared,
    now: fn() -> Rfc3339,
    sessions: Arc<AtomicUsize>,
    path: String,
) {
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("{} console socket: {error}", now());
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        };

        if sessions.load(Ordering::SeqCst) >= MAX_SESSIONS {
            refuse(&stream);
            continue;
        }

        sessions.fetch_add(1, Ordering::SeqCst);
        let shared = shared.clone();
        let sessions_for_thread = Arc::clone(&sessions);
        let path = path.clone();
        let spawned = thread::Builder::new()
            .name("vigil-console".to_string())
            .spawn(move || {
                session(stream, &shared, now, &path);
                sessions_for_thread.fetch_sub(1, Ordering::SeqCst);
            });

        if spawned.is_err() {
            sessions.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

fn session(stream: UnixStream, shared: &Shared, now: fn() -> Rfc3339, path: &str) {
    let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));

    let mut reader = match stream.try_clone() {
        Ok(clone) => BufReader::new(clone),
        Err(error) => {
            eprintln!("{} console socket {path}: {error}", now());
            return;
        }
    };
    let mut stream = stream;

    if let Err(error) = serve(&mut reader, &mut stream, shared, &now)
        && error.kind() != std::io::ErrorKind::BrokenPipe
    {
        eprintln!("{} console socket {path}: {error}", now());
    }
}

fn refuse(stream: &UnixStream) {
    let refusal = Response::Error {
        error: ProtocolError::new(
            ProtocolError::TOO_MANY_SESSIONS,
            format!("at most {MAX_SESSIONS} consoles are answered at a time"),
        ),
    };
    let mut stream = stream;
    let _ = stream.write_all(refusal.to_line().as_bytes());
    let _ = stream.flush();
}

#[cfg(test)]
mod tests_directory {
    use super::*;

    #[cfg(target_os = "linux")]
    fn directory(name: &str, mode: u32) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "vigil-listener-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&path).expect("temp dir");
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("chmod");
        path
    }

    #[cfg(target_os = "linux")]
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

    #[cfg(target_os = "linux")]
    #[test]
    fn our_own_uid_is_readable_on_this_host() {
        assert_ne!(
            current_uid(),
            u32::MAX,
            "reading /proc/self/status failed, so every directory would look like somebody else's"
        );
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
