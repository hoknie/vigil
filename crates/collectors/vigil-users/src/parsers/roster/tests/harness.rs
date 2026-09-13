use std::collections::BTreeMap;

use vigil_model::Snapshot;

use super::super::key_file::UserKeyFile;
use super::super::reading::AccountsReading;
use super::super::snapshot::accounts_snapshot;
use crate::parsers::authorized_keys::parse_authorized_keys;
use crate::parsers::group::{GroupEntry, parse_group};
use crate::parsers::shadow::{ShadowFacts, parse_shadow};
use crate::parsers::sudoers::{SudoGrant, parse_sudoers};
use vigil_collect::{PasswdEntry, parse_passwd_entries};

pub(super) const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin
deploy:x:1000:1000:deploy:/home/deploy:/bin/bash
";
pub(super) const GROUP: &str = "\
root:x:0:
sudo:x:27:deploy
docker:x:998:
deploy:x:1000:
";
pub(super) const SHADOW: &str = "\
root:$6$salt$hash:19000:0:99999:7:::
www-data:*:19000:0:99999:7:::
deploy:!$6$salt$hash:19100:0:99999:7:::
";
pub(super) const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr deploy@builder";

pub(super) struct Parts {
    pub(super) passwd: Vec<PasswdEntry>,
    pub(super) groups: Vec<GroupEntry>,
    pub(super) shadow: BTreeMap<String, ShadowFacts>,
    pub(super) sudo: Vec<SudoGrant>,
    pub(super) keys: Vec<UserKeyFile>,
}

pub(super) fn reading_parts() -> Parts {
    Parts {
        passwd: parse_passwd_entries(PASSWD),
        groups: parse_group(GROUP),
        shadow: parse_shadow(SHADOW),
        sudo: parse_sudoers("deploy ALL=(ALL) NOPASSWD: ALL\n", "/etc/sudoers.d/deploy").grants,
        keys: vec![UserKeyFile {
            user: "deploy".into(),
            uid: 1000,
            path: "/home/deploy/.ssh/authorized_keys".into(),
            readable: true,
            keys: parse_authorized_keys(KEY),
        }],
    }
}

pub(super) fn snapshot_of(shadow: Option<&BTreeMap<String, ShadowFacts>>) -> Snapshot {
    let parts = reading_parts();
    accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &parts.passwd,
            groups: &parts.groups,
            shadow,
            sudo: &parts.sudo,
            keys: &parts.keys,
            sessions: &[],
            session_sources: &[],
        },
    )
}
