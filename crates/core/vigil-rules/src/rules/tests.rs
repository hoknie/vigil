use vigil_model::Change;

use super::verdict::{files, launches, persistence, processes, resources};
use crate::fixture;

#[test]
fn no_family_of_rules_reads_the_changes_of_another() {
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
        (
            persistence as fn(&Change) -> Vec<(String, String)>,
            "persistence",
        ),
        (processes, "processes"),
        (launches, "launches"),
        (resources, "resources"),
        (files, "files"),
    ] {
        let mine = match name {
            "persistence" => &unit,
            "processes" => &program,
            "launches" => &launch,
            "files" => &watched,
            _ => &filesystem,
        };
        for change in [&unit, &program, &launch, &filesystem, &watched] {
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
