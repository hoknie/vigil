use vigil_model::Snapshot;

use vigil_collect::{hex, sha256};

use crate::parsers::{
    FilesReading, Found, WalkRow, Walked, WatchedDirectory, WatchedFile, files_snapshot,
};

const SSHD_CONFIG: &str = "Port 22\nPermitRootLogin no\nPasswordAuthentication no\n";

const HOSTS: &str = "127.0.0.1 localhost\n::1 localhost ip6-localhost\n";

const BUNDLE: &str = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n";

const DROPPED_IN: &str = "PasswordAuthentication no\n";

const CEILING_BYTES: u64 = 1024 * 1024;

const A_BIGGER_CEILING: u64 = 8 * 1024 * 1024;

const TREE: &str = "/etc/ssh/sshd_config.d";

const MASK: &str = "/etc/ssh/ssh_config.d/*.conf";

pub fn files() -> Snapshot {
    let files = vec![
        watched("/etc/ssh/sshd_config", SSHD_CONFIG, "0600"),
        watched("/etc/hosts", HOSTS, "0644"),
        WatchedFile {
            ceiling_bytes: A_BIGGER_CEILING,
            ..watched("/etc/ssl/certs/ca-certificates.crt", BUNDLE, "0644")
        },
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
            ceiling_bytes: CEILING_BYTES,
            found: Found::default(),
        },
        WatchedFile {
            digest: None,
            size: 0,
            found: Found {
                kind: Some("directory"),
                ..Found::default()
            },
            ..watched(TREE, "", "0755")
        },
        WatchedFile {
            found: walked("file", None),
            ..watched(
                "/etc/ssh/sshd_config.d/50-cloud-init.conf",
                DROPPED_IN,
                "0600",
            )
        },
        WatchedFile {
            digest: None,
            size: 22,
            found: walked("symlink", Some("/usr/share/ssh/hardening.conf")),
            ..watched("/etc/ssh/sshd_config.d/60-hardening.conf", "", "0777")
        },
    ];
    let directories = vec![
        directory("/usr/local/bin", "0755"),
        directory("/usr/bin", "0755"),
    ];
    let walks = vec![
        WalkRow {
            entry: TREE.into(),
            kind: "tree",
            matched: 2,
            complete: true,
            not_entered: Vec::new(),
            max_file_size: CEILING_BYTES,
        },
        WalkRow {
            entry: MASK.into(),
            kind: "mask",
            matched: 0,
            complete: true,
            not_entered: vec!["/etc/ssh/ssh_config.d/mnt (nfs4)".into()],
            max_file_size: CEILING_BYTES,
        },
    ];

    files_snapshot(
        "2026-09-09T09:00:00.000Z",
        &FilesReading {
            files: &files,
            directories: &directories,
            walks: &walks,
        },
    )
}

fn walked(kind: &'static str, target: Option<&str>) -> Found {
    Found {
        kind: Some(kind),
        target: target.map(str::to_string),
        walked: Some(Walked {
            by: TREE.into(),
            complete: true,
        }),
    }
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
        ceiling_bytes: CEILING_BYTES,
        found: Found::default(),
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
