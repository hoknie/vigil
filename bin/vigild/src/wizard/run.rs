use std::path::Path;

use super::{configuration, take, write};
use crate::Config;
use crate::helpers::rfc3339;

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
    let survey = take(&Config::default())?;
    let text = configuration(&rfc3339::now(), &survey, &Config::default());

    let watching: Vec<&str> = survey
        .iter()
        .filter(|collector| collector.runs_here())
        .map(|collector| collector.name.as_str())
        .collect();

    if options.dry_run {
        println!("{text}");
        return Ok(format!(
            "nothing was written (--dry-run). It would watch: {}",
            join(&watching)
        ));
    }

    let written = write(Path::new(&options.path), &text, options.force)?;

    let mut said = format!("wrote {} (0600)", written.path.display());
    if let Some(previous) = &written.previous {
        said.push_str(&format!(
            "\n  what was there before is kept as {}",
            previous.display()
        ));
    }
    said.push_str(&format!("\n  watching: {}", join(&watching)));
    for collector in survey.iter().filter(|collector| !collector.runs_here()) {
        said.push_str(&format!(
            "\n  not watching: {} — {}",
            collector.name,
            collector.reason().unwrap_or("no reason was given")
        ));
    }
    said.push_str("\n\nNext:\n  systemctl enable --now vigild\n  vigil");
    Ok(said)
}

fn join(names: &[&str]) -> String {
    match names.is_empty() {
        true => "nothing: no collector on this list can run on this host".to_string(),
        false => names.join(", "),
    }
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

    #[test]
    fn a_host_where_nothing_runs_says_so_rather_than_listing_an_empty_set() {
        assert!(join(&[]).contains("nothing"));
        assert_eq!(join(&["ports", "users"]), "ports, users");
    }
}
