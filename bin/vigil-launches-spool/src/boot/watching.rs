use std::io::{BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};

const PATH: &str = "/usr/sbin:/usr/bin:/sbin:/bin";

const ASKED: &[&str] = &["exec", "--format", "json"];

const COMPLAINT_CEILING: u64 = 8 * 1024;

pub struct Watching {
    pub child: Child,
    pub complaint: JoinHandle<String>,
}

pub fn started(program: &str) -> Result<Watching, String> {
    let mut child = Command::new(program)
        .args(ASKED)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .env("PATH", PATH)
        .env("LC_ALL", "C")
        .spawn()
        .map_err(|error| format!("{program} could not be run: {error}"))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("{program} was started with no standard error"))?;
    let complaint = thread::spawn(move || {
        let mut kept = Vec::new();
        let mut stream = BufReader::new(stderr);
        let _ = (&mut stream).take(COMPLAINT_CEILING).read_to_end(&mut kept);
        let _ = std::io::copy(&mut stream, &mut std::io::sink());
        String::from_utf8_lossy(&kept).trim().to_string()
    });

    Ok(Watching { child, complaint })
}

pub fn arguments() -> String {
    ASKED.join(" ")
}
