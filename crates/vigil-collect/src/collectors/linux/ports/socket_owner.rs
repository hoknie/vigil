use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;

use crate::parsers::{ProcessOwner, redact};

pub struct OwnerScan {
    pub by_inode: BTreeMap<u64, ProcessOwner>,
    pub processes_seen: usize,
    pub processes_denied: usize,
}

pub fn resolve_owners(inodes: &BTreeSet<u64>) -> OwnerScan {
    let mut scan = OwnerScan {
        by_inode: BTreeMap::new(),
        processes_seen: 0,
        processes_denied: 0,
    };

    let Ok(entries) = fs::read_dir("/proc") else {
        return scan;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        scan.processes_seen += 1;

        let descriptors = match fs::read_dir(format!("/proc/{pid}/fd")) {
            Ok(descriptors) => descriptors,
            Err(error) => {
                if error.kind() == ErrorKind::PermissionDenied {
                    scan.processes_denied += 1;
                }
                continue;
            }
        };

        for descriptor in descriptors.flatten() {
            let Ok(target) = fs::read_link(descriptor.path()) else {
                continue;
            };
            let Some(inode) = socket_inode(&target.to_string_lossy()) else {
                continue;
            };
            if !inodes.contains(&inode) || scan.by_inode.contains_key(&inode) {
                continue;
            }
            scan.by_inode.insert(inode, describe(pid));
        }
    }

    scan
}

fn socket_inode(target: &str) -> Option<u64> {
    target
        .strip_prefix("socket:[")?
        .strip_suffix(']')?
        .parse()
        .ok()
}

fn describe(pid: u32) -> ProcessOwner {
    let mut owner = ProcessOwner::default();

    if let Ok(metadata) = fs::metadata(format!("/proc/{pid}")) {
        owner.uid = Some(metadata.uid());
    }

    if let Ok(executable) = fs::read_link(format!("/proc/{pid}/exe")) {
        let path = executable.to_string_lossy().to_string();
        match path.strip_suffix(" (deleted)") {
            Some(real) => {
                owner.executable = Some(real.to_string());
                owner.executable_deleted = true;
            }
            None => owner.executable = Some(path),
        }
    }

    if let Ok(raw) = fs::read(format!("/proc/{pid}/cmdline")) {
        let arguments: Vec<String> = raw
            .split(|byte| *byte == 0)
            .filter(|part| !part.is_empty())
            .map(|part| String::from_utf8_lossy(part).to_string())
            .collect();
        if !arguments.is_empty() {
            let clean = redact(&arguments);
            owner.command_line = Some(clean.text);
            owner.command_line_redacted = clean.redacted;
        }
    }

    owner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_a_socket_link_and_ignores_everything_else() {
        assert_eq!(socket_inode("socket:[20481]"), Some(20481));
        assert_eq!(socket_inode("/var/log/nginx/access.log"), None);
        assert_eq!(socket_inode("anon_inode:[eventpoll]"), None);
        assert_eq!(socket_inode("socket:[]"), None);
    }
}
