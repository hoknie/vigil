use std::path::Path;

use vigil_config::write;

use super::plan::{Planned, absolute, directory_of, plan};
use super::progress::{self, Action};
use super::prose::wrap;
use super::{Surveyed, take};
use crate::Config;

pub const DEFAULT_PATH: &str = "/etc/vigil/vigil.yaml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub path: String,
    pub force: bool,
    pub dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            path: DEFAULT_PATH.to_string(),
            force: false,
            dry_run: false,
        }
    }
}

pub fn configure(options: &Options) -> Result<String, String> {
    eprintln!("vigild configure: asking each collector what it can read on this host");
    let survey = take(&Config::default())?;
    for line in progress::survey(&survey) {
        eprintln!("{line}");
    }
    configure_with(options, &survey)
}

pub fn configure_with(options: &Options, survey: &[Surveyed]) -> Result<String, String> {
    let configuration = absolute(&options.path);
    let planned = plan(&configuration, survey);

    if options.dry_run {
        for file in &planned {
            if let Some(text) = &file.text {
                println!("==> {} <==\n{text}", file.path.display());
            }
        }
        eprintln!("vigild configure: --dry-run, so nothing was written. A run would:");
        for file in &planned {
            let action = Action::of(file, there(file).as_deref(), options.force);
            for line in progress::said(action, file, true) {
                eprintln!("{line}");
            }
        }
        return Ok("nothing was written (--dry-run)".to_string());
    }

    eprintln!("vigild configure: writing the configuration of this host");
    let mut untouched = 0;
    let mut configuration_written = false;
    for file in &planned {
        let action = Action::of(file, there(file).as_deref(), options.force);
        carry_out(action, file)?;
        for line in progress::said(action, file, false) {
            eprintln!("{line}");
        }
        if action.touches_nothing_that_is_there() {
            untouched += 1;
        }
        if file.path == configuration && matches!(action, Action::Write | Action::Replace) {
            configuration_written = true;
        }
    }

    let path = configuration.display().to_string();
    match configuration_written {
        true => {
            crate::config::load(&path)
                .map_err(|error| format!("what was written would not start the daemon: {error}"))?;
        }
        false => {
            for line in unread(&configuration) {
                eprintln!("{line}");
            }
        }
    }

    if untouched > 0 {
        return Err(format!(
            "{untouched} file(s) already there were not touched.\n  \
             --force    replace each one; what is there now is kept beside it as .previous\n  \
             --dry-run  print what would be written, and write nothing"
        ));
    }
    Ok("done. Next:\n  systemctl enable --now vigild\n  vigil ui".to_string())
}

fn there(file: &Planned) -> Option<String> {
    match std::fs::read(&file.path) {
        Ok(bytes) => Some(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => None,
    }
}

fn carry_out(action: Action, file: &Planned) -> Result<(), String> {
    match (action, &file.text) {
        (Action::Write | Action::Replace, Some(text)) => {
            write(&file.path, text, action == Action::Replace).map(|_| ())
        }
        (Action::SetAside, _) => {
            let mut aside = file.path.clone().into_os_string();
            aside.push(".previous");
            std::fs::rename(&file.path, &aside)
                .map_err(|error| format!("{}: {error}", file.path.display()))
        }
        _ => Ok(()),
    }
}

fn unread(configuration: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(configuration) else {
        return Vec::new();
    };
    let Ok(None) = vigil_config::collectors_path(configuration, &text) else {
        return Vec::new();
    };
    wrap(
        &format!(
            "{} names no collectors_path, so the files under collectors/ are not read until it \
             does: add `collectors_path: {}` to it",
            configuration.display(),
            directory_of(configuration).join("collectors").display()
        ),
        progress::WIDTH - 2,
    )
    .into_iter()
    .map(|line| format!("  {line}"))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_the_path_the_packages_install_to() {
        assert_eq!(Options::default().path, DEFAULT_PATH);
        assert!(!Options::default().force);
        assert!(!Options::default().dry_run);
    }
}
