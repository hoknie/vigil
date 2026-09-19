use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use vigil_model::Controlling;

const PLACES: &[&str] = crate::collector::SYSTEMCTL_PLACES;

const WAIT: Duration = Duration::from_secs(30);

const LOOK: Duration = Duration::from_millis(50);

pub fn run(controlling: Controlling, unit: &str) -> Result<String, String> {
    let word = word(controlling)?;
    let Some(path) = found() else {
        return Err(format!(
            "systemctl is not on this host (looked in {}), so nothing here starts or stops a \
             unit",
            PLACES.join(", ")
        ));
    };

    let mut child = Command::new(path)
        .args([word, unit])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .env_clear()
        .env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin")
        .env("LC_ALL", "C")
        .spawn()
        .map_err(|error| format!("{path} {word} {unit} could not be run: {error}"))?;

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Err(error) => return Err(format!("{path} {word} {unit}: {error}")),
            Ok(Some(status)) => {
                let said = complaint(&mut child);
                return match (status.success(), said.is_empty()) {
                    (true, _) => Ok(format!("{path} {word} {unit}")),
                    (false, true) => Err(format!(
                        "systemctl {word} {unit} answered {status} and said nothing"
                    )),
                    (false, false) => Err(format!("systemctl {word} {unit} refused: {said}")),
                };
            }
            Ok(None) if started.elapsed() >= WAIT => {
                let _ = child.kill();
                return Err(format!(
                    "systemctl {word} {unit} had not finished after {} seconds and was \
                     stopped; the unit may still be on its way, and the next reading says",
                    WAIT.as_secs()
                ));
            }
            Ok(None) => std::thread::sleep(LOOK),
        }
    }
}

fn word(controlling: Controlling) -> Result<&'static str, String> {
    match controlling {
        Controlling::Stop => Ok("stop"),
        Controlling::Start => Ok("start"),
        Controlling::Disable => Ok("disable"),
        Controlling::Enable => Ok("enable"),
        Controlling::Mask => Ok("mask"),
        Controlling::Unmask => Ok("unmask"),
        Controlling::Comment | Controlling::Uncomment => Err(format!(
            "{} is what this console does to a line of a crontab; systemd has no word for it",
            controlling.as_str()
        )),
    }
}

fn found() -> Option<&'static str> {
    PLACES.iter().copied().find(|path| Path::new(path).exists())
}

fn complaint(child: &mut std::process::Child) -> String {
    use std::io::Read;

    let mut said = String::new();
    if let Some(stderr) = child.stderr.as_mut() {
        let _ = stderr.read_to_string(&mut said);
    }
    said.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_six_words_offered_are_the_six_words_systemctl_is_given_and_there_is_no_seventh() {
        let mut said: Vec<&str> = Vec::new();
        for controlling in Controlling::ALL {
            match word(*controlling) {
                Ok(one) => said.push(one),
                Err(refused) => assert!(
                    refused.contains("crontab"),
                    "{}: a way with no word here would be a way this daemon cannot carry \
                     out and does not say so: {refused}",
                    controlling.as_str()
                ),
            }
        }

        assert_eq!(
            said,
            vec!["stop", "start", "disable", "enable", "mask", "unmask"],
            "these six are the whole of what this agent asks systemctl for, each with its \
             opposite beside it. restart, reload, isolate, daemon-reload, poweroff and the \
             rest are not refused by a check somewhere later: they have no word here, so \
             there is no shape of a request that reaches them"
        );
    }

    #[test]
    fn the_argument_list_is_the_word_and_the_unit_and_never_a_flag_or_a_shell() {
        let source = include_str!("systemctl.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the code above the tests of it");

        assert!(
            source.contains(".args([word, unit])"),
            "the whole argument list is built here, from a word of a closed list and a name \
             this agent read off the host. A flag added to it is a flag somebody else chose"
        );
        for through in ["\"sh\"", "/bin/sh", "\"--now\"", "\"--force\""] {
            assert!(
                !source.contains(through),
                "{through} appears in the one place that starts systemctl: a flag the \
                 operator never chose, or a shell that would read the unit name as syntax"
            );
        }
    }

    #[test]
    fn systemctl_is_looked_for_by_absolute_path_and_never_through_the_environment() {
        for place in PLACES {
            assert!(
                place.starts_with('/') && place.ends_with("/systemctl"),
                "{place}: a bare name is looked up in PATH, and this daemon runs as root"
            );
        }
    }

    #[test]
    fn a_host_without_systemctl_is_told_where_it_was_looked_for_rather_than_left_guessing() {
        if found().is_some() {
            return;
        }

        let complaint = run(Controlling::Stop, "nginx.service").expect_err("there is none here");

        assert!(complaint.contains("/usr/bin/systemctl"), "{complaint}");
    }
}
