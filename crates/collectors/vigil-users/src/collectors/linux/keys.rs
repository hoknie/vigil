use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use vigil_collect::PasswdEntry;

use crate::parsers::{UserKeyFile, parse_authorized_keys};

const KEY_FILES: &[&str] = &["authorized_keys", "authorized_keys2"];

pub fn read_authorized_keys(passwd: &[PasswdEntry]) -> Vec<UserKeyFile> {
    let mut files = Vec::new();

    for entry in passwd {
        if entry.home.is_empty() {
            continue;
        }
        let directory = Path::new(&entry.home).join(".ssh");

        match fs::metadata(&directory) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => continue,
            Err(error)
                if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
            {
                continue;
            }
            Err(_) => {
                files.push(unreadable(entry, &directory));
                continue;
            }
        }

        let mut refused = false;
        for file_name in KEY_FILES {
            let path = directory.join(file_name);
            match fs::read_to_string(&path) {
                Ok(text) => files.push(UserKeyFile {
                    user: entry.name.clone(),
                    uid: entry.uid,
                    path: path.to_string_lossy().into_owned(),
                    readable: true,
                    keys: parse_authorized_keys(&text),
                }),
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(_) => refused = true,
            }
        }
        if refused {
            files.push(unreadable(entry, &directory));
        }
    }

    files
}

fn unreadable(entry: &PasswdEntry, directory: &Path) -> UserKeyFile {
    UserKeyFile {
        user: entry.name.clone(),
        uid: entry.uid,
        path: directory.to_string_lossy().into_owned(),
        readable: false,
        keys: Vec::new(),
    }
}
