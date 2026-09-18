use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use vigil_engines::{ANSWERED, Answer, Asked, FAILED, TIMED_OUT};

const PATH: &str = "/usr/sbin:/usr/bin:/sbin:/bin";

const LOOK: Duration = Duration::from_millis(25);

const COMPLAINT_CEILING: u64 = 8 * 1024;

pub fn ask(program: &str, asked: &Asked, deadline: Duration, ceiling: u64, at: &Path) -> Answer {
    let arguments: Vec<String> = asked
        .arguments
        .iter()
        .map(|word| (*word).to_string())
        .collect();
    let printed = scratch(at, asked, "out");
    let complained = scratch(at, asked, "err");

    let started = Instant::now();
    let child = start(program, asked, &printed, &complained);
    let mut child = match child {
        Ok(child) => child,
        Err(why) => return unfinished(FAILED, arguments, started, None, Some(why)),
    };

    let state = waited(&mut child, deadline);
    let (said, truncated) = bounded(&printed, ceiling);
    let (why, _) = bounded(&complained, COMPLAINT_CEILING);
    let _ = fs::remove_file(&printed);
    let _ = fs::remove_file(&complained);

    let why = match why.trim().is_empty() {
        true => None,
        false => Some(why.trim().to_string()),
    };

    match state {
        Some(status) if status == Some(0) => Answer {
            state: ANSWERED.to_string(),
            arguments,
            milliseconds: started.elapsed().as_millis() as u64,
            printed: said,
            truncated,
            status,
            why,
        },
        Some(status) => unfinished(FAILED, arguments, started, status, why),
        None => unfinished(TIMED_OUT, arguments, started, None, why),
    }
}

fn start(program: &str, asked: &Asked, printed: &Path, complained: &Path) -> Result<Child, String> {
    let out = File::create(printed).map_err(|error| format!("{}: {error}", printed.display()))?;
    let err =
        File::create(complained).map_err(|error| format!("{}: {error}", complained.display()))?;

    Command::new(program)
        .args(asked.arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .env_clear()
        .env("PATH", PATH)
        .env("LC_ALL", "C")
        .spawn()
        .map_err(|error| format!("{program} could not be run: {error}"))
}

fn waited(child: &mut Child, deadline: Duration) -> Option<Option<i32>> {
    let started = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status.code()),
            Err(_) => return Some(None),
            Ok(None) if started.elapsed() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(LOOK),
        }
    }
}

fn bounded(path: &Path, ceiling: u64) -> (String, bool) {
    let Ok(file) = File::open(path) else {
        return (String::new(), false);
    };
    let over = fs::metadata(path)
        .map(|held| held.len() > ceiling)
        .unwrap_or(false);

    let mut read = Vec::new();
    let _ = file.take(ceiling).read_to_end(&mut read);

    (String::from_utf8_lossy(&read).into_owned(), over)
}

fn scratch(at: &Path, asked: &Asked, extension: &str) -> PathBuf {
    at.join(format!(
        ".vigil-container-dump.{}.{}.{extension}",
        std::process::id(),
        asked.subject.as_str()
    ))
}

fn unfinished(
    state: &str,
    arguments: Vec<String>,
    started: Instant,
    status: Option<i32>,
    why: Option<String>,
) -> Answer {
    Answer {
        state: state.to_string(),
        arguments,
        milliseconds: started.elapsed().as_millis() as u64,
        printed: String::new(),
        truncated: false,
        status,
        why,
    }
}
