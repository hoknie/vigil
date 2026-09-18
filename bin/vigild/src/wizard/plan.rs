use std::path::{Path, PathBuf};

use vigil_config::new_block;

use super::Surveyed;
use super::documents::{kept, names};
use super::placing::placed;
use super::shipped::{COLLECTORS, CONFIGURATION, WATCH_LIST, WATCHED_BY};

const COLLECTORS_DIRECTORY: &str = "collectors";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub path: PathBuf,
    pub text: Option<String>,
    pub holds: Vec<String>,
}

pub fn plan(configuration: &Path, survey: &[Surveyed]) -> Vec<Planned> {
    let directory = directory_of(configuration);
    let runs = |name: &str| {
        survey
            .iter()
            .any(|collector| collector.name == name && collector.runs_here())
    };

    let mut planned = vec![Planned {
        path: configuration.to_path_buf(),
        text: Some(placed(CONFIGURATION, &directory)),
        holds: Vec::new(),
    }];

    for shipped in COLLECTORS {
        planned.push(Planned {
            path: directory.join(shipped.path),
            text: kept(shipped.text, runs).map(|text| placed(&text, &directory)),
            holds: names(shipped.text),
        });
    }

    let shipped: Vec<String> = COLLECTORS
        .iter()
        .flat_map(|shipped| names(shipped.text))
        .collect();
    for collector in survey
        .iter()
        .filter(|collector| collector.runs_here() && !shipped.contains(&collector.name))
    {
        planned.push(Planned {
            path: directory
                .join(COLLECTORS_DIRECTORY)
                .join(format!("{}.yaml", collector.name)),
            text: Some(new_block(
                &collector.name,
                crate::modules::every_seconds_of(&collector.name).unwrap_or(60),
            )),
            holds: vec![collector.name.clone()],
        });
    }

    if runs(WATCHED_BY) {
        planned.push(Planned {
            path: directory.join(WATCH_LIST.path),
            text: Some(placed(WATCH_LIST.text, &directory)),
            holds: Vec::new(),
        });
    }

    planned
}

pub fn directory_of(configuration: &Path) -> PathBuf {
    match configuration.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

pub fn absolute(path: &str) -> PathBuf {
    let path = Path::new(path);
    match path.is_absolute() {
        true => path.to_path_buf(),
        false => std::env::current_dir()
            .map(|here| here.join(path))
            .unwrap_or_else(|_| path.to_path_buf()),
    }
}
