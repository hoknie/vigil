use std::path::Path;

use vigil_config::{Layout, write};

use super::launch_agent;
use super::layout;
use super::plan::{Planned, absolute, directory_of, plan};
use super::progress::{self, Action};
use super::prose::wrap;
use super::{Surveyed, take};
use crate::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub path: String,
    pub force: bool,
    pub dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            path: layout::default_path(),
            force: false,
            dry_run: false,
        }
    }
}

pub fn configure(options: &Options) -> Result<String, String> {
    layout::named(&options.path)?;
    eprintln!("vigild configure: asking each collector what it can read on this host");
    let survey = take(&Config::default())?;
    for line in progress::survey(&survey) {
        eprintln!("{line}");
    }
    let layout = layout::here()?;
    let left = layout::job_left_by_the_archive().filter(|job| job.exists());
    configure_with(&layout, options, &survey, left.as_deref())
}

pub fn configure_with(
    layout: &Layout,
    options: &Options,
    survey: &[Surveyed],
    left_by_the_archive: Option<&Path>,
) -> Result<String, String> {
    layout::named(&options.path)?;
    let configuration = absolute(&options.path);
    let mut planned = plan(layout, &configuration, survey);
    if let Some(job) = launch_agent::planned(layout, &launch_agent::this_program(), &configuration)
    {
        planned.push(job);
    }

    if let Some(job) = left_by_the_archive.filter(|_| under_homebrew(layout)) {
        for line in left_job(job) {
            eprintln!("{line}");
        }
    }

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
    if layout.per_account {
        logs_made(&layout.log_directory)?;
    }
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
    let home = std::env::var("HOME").ok();
    Ok(next_steps(layout, &configuration, home.as_deref()))
}

fn under_homebrew(layout: &Layout) -> bool {
    layout.per_account && layout.launch_agent.is_none()
}

pub(super) fn left_job(job: &Path) -> Vec<String> {
    let name = job
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    wrap(
        &format!(
            "~/Library/LaunchAgents/{} was left by the archive install and is not touched. It starts a second agent \
             on the same socket as the Homebrew service: remove it with `launchctl bootout \
             gui/$(id -u)/{}` and delete the file before `brew services start vigil`",
            name,
            vigil_config::JOB_LABEL
        ),
        progress::WIDTH - 2,
    )
    .into_iter()
    .map(|line| format!("  {line}"))
    .collect()
}

pub(super) fn next_steps(layout: &Layout, configuration: &Path, home: Option<&str>) -> String {
    match layout.per_account {
        false => format!("done. Next:\n  {}\n  vigil ui", layout.start),
        true if under_homebrew(layout) => format!(
            "done. Next:\n  {}\n  or in this terminal:  vigild {}\n  vigil ui",
            layout.start,
            under_tilde(&configuration.display().to_string(), home)
        ),
        true => format!(
            "done. Next:\n  {}\n  or in this terminal:  vigild {}\n  vigil ui",
            under_tilde(&layout.start, home),
            under_tilde(&configuration.display().to_string(), home)
        ),
    }
}

fn under_tilde(text: &str, home: Option<&str>) -> String {
    match home.map(|home| home.trim_end_matches('/')) {
        Some(home) if !home.is_empty() => text.replace(&format!("{home}/"), "~/"),
        _ => text.to_string(),
    }
}

fn logs_made(directory: &str) -> Result<(), String> {
    let path = Path::new(directory);
    if path.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(path).map_err(|error| format!("{directory}: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| format!("{directory}: {error}"))?;
    }
    Ok(())
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
    use vigil_config::Installation;

    fn ann(name: &str) -> Layout {
        Layout::for_account(
            Installation::MACOS,
            &vigil_config::Account {
                uid: 501,
                home: Some(format!("/Users/{name}").into()),
            },
        )
        .expect("a layout")
    }

    #[test]
    fn a_mac_of_an_account_is_told_to_bootstrap_its_gui_domain_and_never_sudo() {
        let layout = ann("ann");
        let said = next_steps(
            &layout,
            Path::new("/Users/ann/.config/vigil/vigil.yaml"),
            Some("/Users/ann"),
        );

        assert_eq!(
            said,
            "done. Next:\n  launchctl bootstrap gui/501 \
             ~/Library/LaunchAgents/vigil.vigild.plist\n  \
             or in this terminal:  vigild ~/.config/vigil/vigil.yaml\n  vigil ui"
        );
        assert!(!said.contains("sudo"));
    }

    #[test]
    fn every_line_of_the_next_step_fits_eighty_columns_for_a_long_home() {
        let name = "a".repeat(32);
        let layout = ann(&name);
        let configuration = format!("/Users/{name}/.config/vigil/vigil.yaml");

        let said = next_steps(
            &layout,
            Path::new(&configuration),
            Some(&format!("/Users/{name}")),
        );

        for line in said.lines() {
            assert!(line.chars().count() <= progress::WIDTH, "{line}");
        }
    }

    #[test]
    fn on_linux_and_as_root_the_next_step_is_the_one_it_always_was() {
        for installation in [Installation::LINUX, Installation::MACOS] {
            let layout = Layout::system(installation);

            assert_eq!(
                next_steps(&layout, Path::new("/etc/vigil/vigil.yaml"), Some("/root")),
                format!("done. Next:\n  {}\n  vigil ui", installation.service.start)
            );
        }
    }

    #[test]
    fn the_default_is_the_path_the_layout_of_this_account_names() {
        assert_eq!(Options::default().path, layout::default_path());
        assert!(!Options::default().force);
        assert!(!Options::default().dry_run);
    }
}
