use std::process::{Command, Stdio};
use std::time::Duration;

use super::manager::Manager;

pub const SYSTEMCTL: &str = "/usr/bin/systemctl";

pub const SYSTEMCTL_PLACES: &[&str] = &[SYSTEMCTL, "/bin/systemctl"];

pub const LAUNCHCTL: &str = "/bin/launchctl";

const WAIT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    NoManager(String),
    Masked,
    Known(String),
}

pub fn program(manager: Manager) -> &'static str {
    match manager {
        Manager::Systemd => SYSTEMCTL_PLACES
            .iter()
            .copied()
            .find(|place| std::path::Path::new(place).is_file())
            .unwrap_or(SYSTEMCTL),
        Manager::Launchd => LAUNCHCTL,
    }
}

pub fn standing(unit: &str) -> Standing {
    let manager = Manager::here();
    if let Some(why) = manager.missing(unit) {
        return Standing::NoManager(why);
    }
    let (answered, said) = asked(program(manager), &manager.asking(unit));
    match manager {
        Manager::Systemd => match said.trim() {
            "masked" | "masked-runtime" => Standing::Masked,
            other => Standing::Known(other.to_string()),
        },
        Manager::Launchd => Standing::Known(
            match answered {
                true => "loaded",
                false => "not loaded",
            }
            .to_string(),
        ),
    }
}

pub fn said_enabling(unit: &str) -> String {
    let manager = Manager::here();
    spelled(program(manager), &manager.enabling(unit, false))
}

pub fn said_disabling(unit: &str) -> String {
    let manager = Manager::here();
    spelled(program(manager), &manager.disabling(unit, true))
}

pub fn enable(unit: &str) -> Result<String, String> {
    let manager = Manager::here();
    let loaded = standing(unit) == Standing::Known("loaded".to_string());
    run_each(program(manager), &manager.enabling(unit, loaded))
}

pub fn disable(unit: &str) -> Result<String, String> {
    let manager = Manager::here();
    let loaded = standing(unit) == Standing::Known("loaded".to_string());
    run_each(program(manager), &manager.disabling(unit, loaded))
}

fn spelled(program: &str, steps: &[Vec<String>]) -> String {
    steps
        .iter()
        .map(|arguments| format!("{program} {}", arguments.join(" ")))
        .collect::<Vec<String>>()
        .join("; ")
}

fn run_each(program: &str, steps: &[Vec<String>]) -> Result<String, String> {
    let mut done = Vec::new();
    for arguments in steps {
        done.push(run(program, arguments)?);
    }
    Ok(done.join("; "))
}

fn run(program: &str, arguments: &[String]) -> Result<String, String> {
    let said = format!("{program} {}", arguments.join(" "));
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("{said}: {error}"))?;

    let outcome = match child
        .wait_timeout(WAIT)
        .map_err(|error| format!("{said}: {error}"))?
    {
        Some(outcome) => outcome,
        None => {
            let _ = child.kill();
            return Err(format!(
                "{said} did not finish within {} seconds and was stopped",
                WAIT.as_secs()
            ));
        }
    };

    match outcome.status.success() {
        true => Ok(said),
        false => Err(format!(
            "{said} answered {}: {}",
            outcome.status,
            outcome.said.trim()
        )),
    }
}

fn asked(program: &str, arguments: &[String]) -> (bool, String) {
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|out| {
            (
                out.status.success(),
                String::from_utf8_lossy(&out.stdout).to_string(),
            )
        })
        .unwrap_or((false, String::new()))
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
    use std::path::Path;

    use super::*;

    #[test]
    fn the_only_programs_this_product_starts_are_named_by_their_absolute_paths() {
        for manager in [Manager::Systemd, Manager::Launchd] {
            assert!(
                Path::new(program(manager)).is_absolute(),
                "a bare name is looked up in PATH, and what PATH means for a daemon is not what \
                 it means in the shell an operator tested it in"
            );
        }
        assert_eq!(SYSTEMCTL, "/usr/bin/systemctl");
        assert_eq!(
            LAUNCHCTL, "/bin/launchctl",
            "launchctl lives on the sealed system volume of every Mac, at this one path"
        );
    }

    #[test]
    fn a_host_with_no_service_manager_for_it_is_told_apart_from_a_unit_nobody_enabled() {
        match standing("vigil-nothing-of-ours.timer") {
            Standing::NoManager(why) => assert!(!why.is_empty()),
            Standing::Known(said) => assert_ne!(
                said, "masked",
                "a unit nobody installed came back masked, so the two answers are not \
                 being told apart"
            ),
            Standing::Masked => panic!("a unit that does not exist on this host is not masked"),
        }
    }

    #[test]
    fn what_a_dry_run_would_run_is_every_step_by_the_absolute_path_of_its_program() {
        let said = said_enabling("vigil-firewall.timer");

        assert!(said.starts_with(program(Manager::here())), "{said}");
        match Manager::here() {
            Manager::Systemd => {
                assert_eq!(
                    said,
                    "/usr/bin/systemctl enable --now vigil-firewall.timer; /usr/bin/systemctl \
                     start --no-block vigil-firewall.service"
                )
            }
            Manager::Launchd => assert_eq!(
                said,
                "/bin/launchctl enable system/vigil.firewall; /bin/launchctl bootstrap system \
                 /Library/LaunchDaemons/vigil.firewall.plist"
            ),
        }
    }
}
