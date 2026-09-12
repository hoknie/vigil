use vigil_model::Snapshot;

use crate::helpers::{hex, sha256};
use crate::parsers::{FilesReading, WatchedDirectory, WatchedFile, files_snapshot};

const SSHD_CONFIG: &str = "Port 22\nPermitRootLogin no\nPasswordAuthentication no\n";

const HOSTS: &str = "127.0.0.1 localhost\n::1 localhost ip6-localhost\n";

pub fn files() -> Snapshot {
    let files = vec![
        watched("/etc/ssh/sshd_config", SSHD_CONFIG, "0600"),
        watched("/etc/hosts", HOSTS, "0644"),
        WatchedFile {
            path: "/etc/pam.d/sshd".into(),
            present: false,
            readable: false,
            digest: None,
            size: 0,
            mode: None,
            uid: None,
            gid: None,
            over_the_ceiling: false,
        },
    ];
    let directories = vec![
        directory("/usr/local/bin", "0755"),
        directory("/usr/bin", "0755"),
    ];

    files_snapshot(
        "2026-09-09T09:00:00.000Z",
        &FilesReading {
            files: &files,
            directories: &directories,
        },
    )
}

fn watched(path: &str, content: &str, mode: &str) -> WatchedFile {
    WatchedFile {
        path: path.to_string(),
        present: true,
        readable: true,
        digest: Some(hex(&sha256(content.as_bytes()))),
        size: content.len() as u64,
        mode: Some(mode.to_string()),
        uid: Some(0),
        gid: Some(0),
        over_the_ceiling: false,
    }
}

fn directory(path: &str, mode: &str) -> WatchedDirectory {
    WatchedDirectory {
        path: path.to_string(),
        present: true,
        mode: Some(mode.to_string()),
        uid: Some(0),
        gid: Some(0),
    }
}
