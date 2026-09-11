use vigil_model::Change;

use super::fixture;
use super::verdict::{accounts, firewall, launches, persistence, ports, processes};

#[test]
fn the_six_families_of_rules_do_not_read_each_others_changes() {
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

    for (family, name) in [
        (ports as fn(&Change) -> Vec<(String, String)>, "sockets"),
        (accounts, "accounts"),
        (persistence, "persistence"),
        (processes, "processes"),
        (launches, "launches"),
        (firewall, "firewall"),
    ] {
        let mine = match name {
            "sockets" => &socket,
            "accounts" => &account,
            "persistence" => &unit,
            "processes" => &program,
            "launches" => &launch,
            _ => &ruleset,
        };
        for change in [&socket, &account, &unit, &program, &launch, &ruleset] {
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
