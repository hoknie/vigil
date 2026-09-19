use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use vigil_collect::parse_passwd_entries;

use crate::parsers::{ScriptFamily, WatchedScript};
use vigil_collect::{absent, hex, sha256, shown_to_the_agent};

use super::files::{read_capped, sorted_files};
use super::{
    BOOT_DIRECTORIES, BOOT_SCRIPTS, NON_INTERACTIVE_SHELLS, PROFILE_DIRECTORIES, SYSTEM_PROFILES,
    USER_PROFILES,
};

pub(super) fn read_scripts() -> Vec<WatchedScript> {
    let mut scripts = Vec::new();

    for path in BOOT_SCRIPTS {
        scripts.push(describe_script(Path::new(path), ScriptFamily::Boot));
    }
    for directory in BOOT_DIRECTORIES {
        for path in sorted_files(Path::new(directory)) {
            scripts.push(describe_script(&path, ScriptFamily::Boot));
        }
    }
    for path in SYSTEM_PROFILES {
        scripts.push(describe_script(Path::new(path), ScriptFamily::Profile));
    }
    for directory in PROFILE_DIRECTORIES {
        for path in sorted_files(Path::new(directory)) {
            scripts.push(describe_script(&path, ScriptFamily::Profile));
        }
    }

    if let Ok(text) = fs::read_to_string("/etc/passwd") {
        for entry in parse_passwd_entries(&text) {
            if entry.home.is_empty() || NON_INTERACTIVE_SHELLS.contains(&entry.shell.as_str()) {
                continue;
            }
            for name in USER_PROFILES {
                let path = Path::new(&entry.home).join(name);
                if path.exists() || !is_shown(&path) {
                    scripts.push(describe_script(&path, ScriptFamily::Profile));
                }
            }
        }
    }

    scripts
}

fn is_shown(path: &Path) -> bool {
    shown_to_the_agent(&path.to_string_lossy())
}

pub(super) fn describe_script(path: &Path, family: ScriptFamily) -> WatchedScript {
    let shown = is_shown(path);
    let mut script = WatchedScript {
        path: path.to_string_lossy().into_owned(),
        family,
        present: shown.then_some(false),
        shown,
        readable: shown.then_some(true),
        digest: None,
        size: 0,
        mode: String::new(),
        uid: 0,
        gid: 0,
    };

    if !shown {
        return script;
    }

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
        Some(text) => script.digest = Some(hex(&sha256(text.as_bytes()))),
        None => script.readable = Some(false),
    }

    script
}
