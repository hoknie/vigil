use vigil_model::Change;

use super::fixture;
use super::verdict::{
    accounts, containers, files, firewall, launches, persistence, ports, processes, resources,
};

#[test]
fn the_nine_families_of_rules_do_not_read_each_others_changes() {
    let socket = Change::Added {
        key: "tcp|0.0.0.0:4444".into(),
        after: fixture::socket("0.0.0.0", 4444, "/tmp/.x/nc", "www-data"),
    };
    let account = Change::Added {
        key: "account|toor".into(),
        after: fixture::account("toor", 0, "/bin/bash"),
    };
    let unit = Change::Added {
        key: "unit|update.service".into(),
        after: fixture::unit("update.service", "/tmp/.x/implant", "root"),
    };
    let program = Change::Added {
        key: "exec|/dev/shm/payload|www-data".into(),
        after: fixture::program("/dev/shm/payload", "www-data", 33, &[]),
    };
    let launch = Change::Added {
        key: "run|alice|/dev/shm/payload".into(),
        after: fixture::launch("alice", 1000, "/dev/shm/payload"),
    };
    let ruleset = Change::Changed {
        key: "fw-summary|nftables".into(),
        before: fixture::firewall_ruleset(2, 3, 14),
        after: fixture::firewall_ruleset(0, 0, 0),
    };
    let container = Change::Added {
        key: "container|3ab1c0f2d4e5".into(),
        after: fixture::container("/usr/local/bin/agent", "000001ffffffffff", &["/"]),
    };
    let watched = Change::Changed {
        key: "file|/etc/ssh/sshd_config".into(),
        before: fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
        after: fixture::watched_file("/etc/ssh/sshd_config", "0600", "b4"),
    };
    let filesystem = Change::Changed {
        key: "fs|/var".into(),
        before: fixture::filesystem("/var", Some(20), Some(85)),
        after: fixture::filesystem("/var", Some(0), Some(85)),
    };

    for (family, name) in [
        (ports as fn(&Change) -> Vec<(String, String)>, "sockets"),
        (accounts, "accounts"),
        (persistence, "persistence"),
        (processes, "processes"),
        (launches, "launches"),
        (firewall, "firewall"),
        (resources, "resources"),
        (containers, "containers"),
        (files, "files"),
    ] {
        let mine = match name {
            "sockets" => &socket,
            "accounts" => &account,
            "persistence" => &unit,
            "processes" => &program,
            "launches" => &launch,
            "firewall" => &ruleset,
            "containers" => &container,
            "files" => &watched,
            _ => &filesystem,
        };
        for change in [
            &socket,
            &account,
            &unit,
            &program,
            &launch,
            &ruleset,
            &filesystem,
            &container,
            &watched,
        ] {
            let fired = family(change);
            match std::ptr::eq(change, mine) {
                true => assert!(!fired.is_empty(), "{name} said nothing about its own"),
                false => assert!(
                    fired.is_empty(),
                    "the {name} rules read {}: {fired:?}",
                    change.key()
                ),
            }
        }
    }
}
