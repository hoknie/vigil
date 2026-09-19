use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use vigil_collect::{absent, hex, sha256};

use super::files::read_capped;
use super::jobs::Home;
use super::places::{PERSONAL_PROFILES, SYSTEM_PROFILES};
use crate::parsers::{ScriptFamily, WatchedScript};

pub(super) fn read_scripts(homes: &[Home]) -> Vec<WatchedScript> {
    let mut scripts: Vec<WatchedScript> = SYSTEM_PROFILES
        .iter()
        .map(|path| described(Path::new(path), ScriptFamily::Profile))
        .collect();

    for home in homes {
        for name in PERSONAL_PROFILES {
            let path = home.path.join(name);
            match fs::symlink_metadata(&path) {
                Err(error) if absent(&error) => continue,
                _ => scripts.push(described(&path, ScriptFamily::Profile)),
            }
        }
    }

    scripts
}

pub(super) fn described(path: &Path, family: ScriptFamily) -> WatchedScript {
    let mut script = WatchedScript {
        path: path.to_string_lossy().into_owned(),
        family,
        present: Some(false),
        shown: true,
        readable: Some(true),
        digest: None,
        size: 0,
        mode: String::new(),
        uid: 0,
        gid: 0,
    };

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            if !absent(&error) {
                script.present = None;
                script.readable = Some(false);
            }
            return script;
        }
    };
    script.present = Some(true);
    script.size = metadata.len();
    script.mode = format!("{:04o}", metadata.permissions().mode() & 0o7777);
    script.uid = metadata.uid();
    script.gid = metadata.gid();

    match read_capped(path) {
        Ok(bytes) => script.digest = Some(hex(&sha256(&bytes))),
        Err(_) => script.readable = Some(false),
    }

    script
}
