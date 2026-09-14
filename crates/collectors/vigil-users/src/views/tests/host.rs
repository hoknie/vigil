use std::collections::BTreeSet;

use serde_json::{Value, json};
use vigil_collect::parse_passwd_entries;
use vigil_model::Snapshot;

use crate::parsers::{
    AccountsReading, Session, UTMP, UserKeyFile, accounts_snapshot, parse_authorized_keys,
    parse_group, parse_sudoers,
};

const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
dep:x:1003:4:dep:/home/dep:/bin/bash
deploy:x:1000:1000:deploy:/home/deploy:/bin/bash
deploy2:x:1002:998:deploy2:/home/deploy2:/bin/bash
contractor:x:1001:1001:contractor:/home/contractor:/bin/bash
www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin
";

const GROUP: &str = "\
root:x:0:
adm:x:4:
sudo:x:27:deploy
docker:x:998:deploy
wheel:x:10:deploy2,contractor
shadow:x:42:someone-gone
deploy:x:1000:
users:x:100:deploy,deploy2,dep
contractor:x:1001:
";

const SUDOERS: &str = "\
%docker ALL=(ALL) NOPASSWD: ALL
%adm ALL=(root) /usr/bin/journalctl
%users ALL=(root) /usr/bin/uptime
deploy2 ALL=(ALL) ALL
contractor ALL=(root) NOPASSWD: /usr/bin/id
%ghost ALL=(ALL) ALL
someone-gone ALL=(ALL) ALL
";

const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

pub(super) const OLDER_AGENT: &str = "account|contractor";

pub(super) fn a_host_with_names_that_prefix_one_another() -> Snapshot {
    let passwd = parse_passwd_entries(PASSWD);
    let groups = parse_group(GROUP);
    let sudo = parse_sudoers(SUDOERS, "/etc/sudoers").grants;
    let keys = vec![
        UserKeyFile {
            user: "deploy".into(),
            uid: 1000,
            path: "/home/deploy/.ssh/authorized_keys".into(),
            readable: true,
            keys: parse_authorized_keys(KEY),
        },
        UserKeyFile {
            user: "deploy2".into(),
            uid: 1002,
            path: "/home/deploy2/.ssh/authorized_keys".into(),
            readable: true,
            keys: parse_authorized_keys(KEY),
        },
        UserKeyFile {
            user: "dep".into(),
            uid: 1003,
            path: "/home/dep/.ssh/authorized_keys".into(),
            readable: false,
            keys: Vec::new(),
        },
        UserKeyFile {
            user: "ghost".into(),
            uid: 1500,
            path: "/home/ghost/.ssh/authorized_keys".into(),
            readable: true,
            keys: parse_authorized_keys(KEY),
        },
    ];
    let sessions = vec![
        Session {
            user: "deploy".into(),
            line: "pts/0".into(),
            pid: 4000,
            sources: BTreeSet::from([UTMP]),
            ..Session::default()
        },
        Session {
            user: "deploy".into(),
            line: "pts/3".into(),
            pid: 4003,
            sources: BTreeSet::from([UTMP]),
            ..Session::default()
        },
        Session {
            user: "deploy2".into(),
            line: "pts/1".into(),
            pid: 4001,
            sources: BTreeSet::from([UTMP]),
            ..Session::default()
        },
        Session {
            user: "dep".into(),
            line: "pts/2".into(),
            pid: 4002,
            sources: BTreeSet::from([UTMP]),
            ..Session::default()
        },
        Session {
            line: "pts/4".into(),
            pid: 4004,
            sources: BTreeSet::from([UTMP]),
            ..Session::default()
        },
    ];

    let mut reading = accounts_snapshot(
        "2026-09-14T09:00:00.000Z",
        &AccountsReading {
            passwd: &passwd,
            groups: &groups,
            shadow: None,
            sudo: &sudo,
            keys: &keys,
            sessions: &sessions,
            session_sources: &[],
        },
    );

    if let Some(account) = reading
        .items
        .get_mut(OLDER_AGENT)
        .and_then(Value::as_object_mut)
    {
        account.remove("groups");
    }
    for (key, item) in [
        ("keyring|root|0x1234", json!({"from": "a newer agent"})),
        ("account", json!({"name": "account"})),
        ("accounting|x", json!({"looks like": "account"})),
    ] {
        reading.items.insert(key.to_string(), item);
    }
    reading
}
