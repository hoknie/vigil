use std::path::{Path, PathBuf};

use crate::helpers::places::{files_in, is_a_directory, pointed_at, read_here};
use crate::helpers::reading::{apart, silenced};
use crate::types::source::Source;

pub const SUPPRESSIONS_PATH: &str = "suppressions_path";

pub const CONSOLE_FILE: &str = "console.yaml";

pub fn suppressions_path(configuration: &Path, text: &str) -> Result<Option<PathBuf>, String> {
    pointed_at(configuration, text, SUPPRESSIONS_PATH)
}

pub fn gathered(path: &Path) -> Result<Vec<Source>, String> {
    files_in(path)?
        .into_iter()
        .map(|file| {
            let text = std::fs::read_to_string(&file)
                .map_err(|error| format!("{}: {error}", file.display()))?;
            let suppressions =
                apart(&text).map_err(|cause| format!("{}: {cause}", file.display()))?;
            Ok(Source {
                path: file,
                suppressions,
            })
        })
        .collect()
}

pub fn sources(configuration: &Path, text: &str) -> Result<Vec<Source>, String> {
    let inline = silenced(text).map_err(|cause| format!("{}: {cause}", configuration.display()))?;
    let mut sources = vec![Source {
        path: configuration.to_path_buf(),
        suppressions: inline,
    }];
    if let Some(path) = suppressions_path(configuration, text)
        .map_err(|cause| format!("{}: {cause}", configuration.display()))?
    {
        sources.extend(gathered(&path)?);
    }
    Ok(sources)
}

pub fn written_to(path: &Path, file: Option<&str>) -> Result<PathBuf, String> {
    match (is_a_directory(path), file) {
        (true, None) => Ok(path.join(CONSOLE_FILE)),
        (true, Some(named)) => {
            let named = match Path::new(named).extension() {
                Some(_) => named.to_string(),
                None => format!("{named}.yaml"),
            };
            let chosen = path.join(&named);
            let plain = Path::new(&named).components().count() == 1;
            match plain && read_here(&chosen) {
                true => Ok(chosen),
                false => Err(format!(
                    "{named} is not a file this agent reads from {}: name one file in it, \
                     ending in .yaml or .yml and not beginning with a dot",
                    path.display()
                )),
            }
        }
        (false, None) => Ok(path.to_path_buf()),
        (false, Some(named)) => Err(format!(
            "{SUPPRESSIONS_PATH} names one file, {}, so there is no directory to put {named} in",
            path.display()
        )),
    }
}
