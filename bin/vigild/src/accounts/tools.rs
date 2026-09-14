use std::path::Path;
use std::process::Command;

use super::utility::Utility;

pub fn run(utility: Utility, arguments: &[String]) -> Result<String, String> {
    let Some(path) = found(utility) else {
        return Err(format!(
            "{} is not on this host (looked in {})",
            utility.name(),
            utility.places().join(", ")
        ));
    };

    let ran = Command::new(path)
        .args(arguments)
        .env_clear()
        .env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin")
        .env("LC_ALL", "C")
        .output()
        .map_err(|error| format!("{path} could not be run: {error}"))?;

    let said = String::from_utf8_lossy(&ran.stderr).trim().to_string();
    match (ran.status.success(), said.is_empty()) {
        (true, _) => Ok(format!("{} finished", utility.name())),
        (false, true) => Err(format!(
            "{} refused, and said nothing ({})",
            utility.name(),
            ran.status
        )),
        (false, false) => Err(format!("{} refused: {said}", utility.name())),
    }
}

fn found(utility: Utility) -> Option<&'static str> {
    utility
        .places()
        .iter()
        .copied()
        .find(|path| Path::new(path).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_without_the_tool_is_told_where_it_was_looked_for_rather_than_left_guessing() {
        if found(Utility::Loginctl).is_some() {
            return;
        }

        let complaint = run(Utility::Loginctl, &["list-sessions".to_string()])
            .expect_err("there is no loginctl here");

        assert!(complaint.contains("/usr/bin/loginctl"), "{complaint}");
    }
}
