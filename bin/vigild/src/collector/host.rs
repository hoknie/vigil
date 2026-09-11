use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub const SYSTEMCTL: &str = "/usr/bin/systemctl";

pub const SYSTEMD: &str = "/run/systemd/system";

const WAIT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    NoSystemd,
    Masked,
    Known(String),
}

pub fn standing(unit: &str) -> Standing {
    if !Path::new(SYSTEMD).is_dir() {
        return Standing::NoSystemd;
    }
    match said(&["is-enabled", unit]).trim() {
        "masked" | "masked-runtime" => Standing::Masked,
        other => Standing::Known(other.to_string()),
    }
}

pub fn enable(unit: &str) -> Result<String, String> {
    run(&["enable", "--now", unit])
}

pub fn disable(unit: &str) -> Result<String, String> {
    run(&["disable", "--now", unit])
}

fn run(arguments: &[&str]) -> Result<String, String> {
    let mut child = Command::new(SYSTEMCTL)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("{SYSTEMCTL} {}: {error}", arguments.join(" ")))?;

    let outcome = match child
        .wait_timeout(WAIT)
        .map_err(|error| format!("{SYSTEMCTL} {}: {error}", arguments.join(" ")))?
    {
        Some(outcome) => outcome,
        None => {
            let _ = child.kill();
            return Err(format!(
                "{SYSTEMCTL} {} did not finish within {} seconds and was stopped",
                arguments.join(" "),
                WAIT.as_secs()
            ));
        }
    };

    match outcome.status.success() {
        true => Ok(format!("{SYSTEMCTL} {}", arguments.join(" "))),
        false => Err(format!(
            "{SYSTEMCTL} {} answered {}: {}",
            arguments.join(" "),
            outcome.status,
            outcome.said.trim()
        )),
    }
}

fn said(arguments: &[&str]) -> String {
    Command::new(SYSTEMCTL)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_default()
}

struct Outcome {
    status: std::process::ExitStatus,
    said: String,
}

trait WaitTimeout {
    fn wait_timeout(&mut self, within: Duration) -> std::io::Result<Option<Outcome>>;
}

impl WaitTimeout for std::process::Child {
    fn wait_timeout(&mut self, within: Duration) -> std::io::Result<Option<Outcome>> {
        let started = std::time::Instant::now();
        loop {
            if let Some(status) = self.try_wait()? {
                let mut said = String::new();
                if let Some(stderr) = self.stderr.as_mut() {
                    use std::io::Read;
                    let _ = stderr.read_to_string(&mut said);
                }
                return Ok(Some(Outcome { status, said }));
            }
            if started.elapsed() >= within {
                return Ok(None);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_only_program_this_product_starts_is_named_by_its_absolute_path() {
        assert!(
            Path::new(SYSTEMCTL).is_absolute(),
            "a bare name is looked up in PATH, and what PATH means for a daemon is not what \
             it means in the shell an operator tested it in"
        );
        assert_eq!(SYSTEMCTL, "/usr/bin/systemctl");
    }

    #[test]
    fn a_host_with_no_systemd_is_told_apart_from_a_unit_nobody_enabled() {
        match standing("vigil-nothing-of-ours.timer") {
            Standing::NoSystemd => {}
            Standing::Known(said) => assert_ne!(
                said, "masked",
                "a unit nobody installed came back masked, so the two answers are not \
                 being told apart"
            ),
            Standing::Masked => panic!("a unit that does not exist on this host is not masked"),
        }
    }
}
