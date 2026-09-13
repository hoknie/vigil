use std::path::Path;

use crate::parsers::{UnitFile, parse_unit};

use super::UNIT_DIRECTORIES;
use super::files::{read_capped, sorted_files};

pub(super) fn read_units() -> Vec<UnitFile> {
    let mut units: Vec<UnitFile> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for directory in UNIT_DIRECTORIES {
        for path in sorted_files(Path::new(directory)) {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.contains('.') || !seen.insert(name.to_string()) {
                continue;
            }

            let text = read_capped(&path);
            units.push(UnitFile {
                name: name.to_string(),
                path: path.to_string_lossy().into_owned(),
                readable: text.is_some(),
                facts: text.map(|text| parse_unit(&text)).unwrap_or_default(),
            });
        }
    }

    units
}
