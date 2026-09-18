use std::path::{Path, PathBuf};

use serde::Deserialize;
use vigil_config::{Source, files_in, gathered, resolved};
use vigil_report::SyslogFacility;

use super::settings::Receiver;

#[derive(Debug, Clone, Default)]
pub struct Apart {
    pub suppressions_at: Option<PathBuf>,
    pub suppressions: Vec<Source>,
    pub reporters_at: Option<PathBuf>,
    pub reporters: Vec<Reporters>,
}

#[derive(Debug, Clone)]
pub struct Reporters {
    pub path: PathBuf,
    pub receivers: Vec<Receiver>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Held {
    #[serde(default)]
    reporters: Vec<Receiver>,
}

pub fn place(
    configuration: &Path,
    key: &str,
    written: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    match written {
        None => Ok(None),
        Some(path) if path.trim().is_empty() => Err(format!(
            "{key} is empty; name a file or a directory, or leave the key out"
        )),
        Some(path) => Ok(Some(resolved(configuration, path))),
    }
}

pub fn suppressions(at: &Path) -> Result<Vec<Source>, String> {
    gathered(at)
}

pub fn reporters(at: &Path) -> Result<Vec<Reporters>, String> {
    files_in(at)?
        .into_iter()
        .map(|file| {
            let text = std::fs::read_to_string(&file)
                .map_err(|error| format!("{}: {error}", file.display()))?;
            let receivers =
                receivers(&text).map_err(|cause| format!("{}: {cause}", file.display()))?;
            Ok(Reporters {
                path: file,
                receivers,
            })
        })
        .collect()
}

fn receivers(text: &str) -> Result<Vec<Receiver>, String> {
    let document: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    if document.is_null() {
        return Ok(Vec::new());
    }
    let held: Held = serde_yaml::from_value(document).map_err(|error| error.to_string())?;

    for (index, receiver) in held.reporters.iter().enumerate() {
        if let Receiver::Syslog { facility } = receiver {
            SyslogFacility::parse(facility)
                .map_err(|cause| format!("reporter #{}: {cause}", index + 1))?;
        }
    }
    Ok(held.reporters)
}
