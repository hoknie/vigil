use std::io::{BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use vigil_model::{ProtocolError, Response, Rfc3339};

use super::bind::bind;
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
