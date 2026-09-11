use std::collections::{BTreeMap, BTreeSet};

use vigil_model::Snapshot;

use crate::parsers::{
    AccountsReading, LOGIND, Session, SessionSource, UTMP, UserKeyFile, accounts_snapshot,
    parse_authorized_keys, parse_group, parse_passwd_entries, parse_shadow, parse_sudoers,
};

const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin
deploy:x:1000:1000:deploy:/home/deploy:/bin/bash
contractor:x:1001:1001:contractor:/home/contractor:/bin/bash
svc-runner:x:998:998:svc-runner:/var/lib/runner:/bin/sh
";

const GROUP: &str = "\
root:x:0:
wheel:x:10:deploy
deploy:x:1000:
users:x:100:
";

const SHADOW: &str = "\
root:*:19000:0:99999:7:::
www-data:!:19000:0:99999:7:::
deploy:$6$salt$hash:19100:0:99999:7:::
contractor:$6$salt$hash:19500:0:90:7::20000:
";

const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

const RESTRICTED_KEY: &str = "from=\"10.0.0.0/8\",no-pty ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr";

pub fn accounts() -> Snapshot {
    let passwd = parse_passwd_entries(PASSWD);
    let groups = parse_group(GROUP);
    let shadow: BTreeMap<_, _> = parse_shadow(SHADOW);
    let sudo = parse_sudoers("%wheel ALL=(ALL) ALL\n", "/etc/sudoers").grants;
    let keys = vec![
        UserKeyFile {
            user: "deploy".into(),
            uid: 1000,
            path: "/home/deploy/.ssh/authorized_keys".into(),
            readable: true,
            keys: parse_authorized_keys(KEY),
        },
        UserKeyFile {
            user: "contractor".into(),
            uid: 1001,
            path: "/home/contractor/.ssh/authorized_keys".into(),
            readable: true,
            keys: parse_authorized_keys(RESTRICTED_KEY),
        },
        UserKeyFile {
            user: "backup".into(),
            uid: 1001,
            path: "/home/backup/.ssh/authorized_keys".into(),
            readable: false,
            keys: Vec::new(),
        },
    ];
    let sessions = vec![
        Session {
            user: "deploy".into(),
            uid: Some(1000),
            line: "pts/0".into(),
            from: "10.0.0.5".into(),
            remote: true,
            pid: 4242,
            id: "83".into(),
            service: "sshd".into(),
            kind: "tty".into(),
            class: "user".into(),
            state: "active".into(),
            sources: BTreeSet::from([LOGIND, UTMP]),
        },
        Session {
            user: "root".into(),
            uid: Some(0),
            line: String::new(),
            from: String::new(),
            remote: false,
            pid: 900,
            id: "84".into(),
            service: "systemd-user".into(),
            kind: "unspecified".into(),
            class: "background".into(),
            state: "closing".into(),
            sources: BTreeSet::from([LOGIND]),
        },
        Session {
            user: "unknown".into(),
            uid: None,
            line: "pts/3".into(),
            from: "203.0.113.9".into(),
            remote: true,
            pid: 5150,
            id: "85".into(),
            service: "sshd".into(),
            kind: "tty".into(),
            class: "user".into(),
            state: "opening".into(),
            sources: BTreeSet::from([UTMP]),
        },
    ];
    let session_sources = vec![
        SessionSource::read(LOGIND, "/run/systemd/sessions", 2),
        SessionSource::absent(
            UTMP,
            "/run/utmp",
            "neither /run/utmp nor /var/run/utmp is on this host: nothing writes a utmp login record here",
        ),
    ];

    accounts_snapshot(
        "2026-09-09T09:00:00.000Z",
        &AccountsReading {
            passwd: &passwd,
            groups: &groups,
            shadow: Some(&shadow),
            sudo: &sudo,
            keys: &keys,
            sessions: &sessions,
            session_sources: &session_sources,
        },
    )
}
