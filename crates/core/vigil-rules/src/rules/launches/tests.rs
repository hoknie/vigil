use serde_json::json;
use vigil_model::Change;

use crate::fixture;
use crate::rules::verdict::launches;

#[test]
fn exactly_one_launch_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "somebody ran a program nobody on this host had run before",
            Change::Added {
                key: "run|alice|/usr/bin/nmap".into(),
                after: fixture::launch("alice", 1000, "/usr/bin/nmap"),
            },
            "first_launch_for_user",
            "exec.first_seen_for_user",
        ),
        (
            "somebody unpacked something into /dev/shm and ran it",
            Change::Added {
                key: "run|alice|/dev/shm/payload".into(),
                after: fixture::launch("alice", 1000, "/dev/shm/payload"),
            },
            "launch_from_writable_path",
            "exec.from_writable_path",
        ),
        (
            "the program was gone from disk by the time the record was read",
            Change::Added {
                key: "run|alice|/opt/build/tool".into(),
                after: fixture::launch_of_a_missing_program("alice", 1000, "/opt/build/tool"),
            },
            "launched_binary_missing",
            "process.binary_deleted",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = launches(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn a_dropper_that_ran_from_tmp_and_deleted_itself_is_still_one_finding() {
    let change = Change::Added {
        key: "run|alice|/tmp/.x/dropper".into(),
        after: fixture::launch_of_a_missing_program("alice", 1000, "/tmp/.x/dropper"),
    };

    let fired = launches(&change);

    assert_eq!(fired.len(), 1, "fired: {fired:?}");
    assert_eq!(fired[0].0, "launch_from_writable_path");
}

#[test]
fn the_rows_saying_the_launch_collector_could_not_see_something_reach_no_rule_at_all() {
    for key in ["launches|unnamed", "launches|capped"] {
        let change = Change::Added {
            key: key.into(),
            after: json!({"named": false, "reason": "…"}),
        };

        assert!(launches(&change).is_empty(), "{key}");
    }
}

#[test]
fn a_launch_snapshot_only_ever_gains_keys_and_the_rules_say_nothing_about_anything_else() {
    let removed = Change::Removed {
        key: "run|alice|/usr/bin/nmap".into(),
        before: fixture::launch("alice", 1000, "/usr/bin/nmap"),
    };
    let changed = Change::Changed {
        key: "run|alice|/usr/bin/nmap".into(),
        before: fixture::launch("alice", 1000, "/usr/bin/nmap"),
        after: fixture::launch_of_a_missing_program("alice", 1000, "/usr/bin/nmap"),
    };

    assert!(launches(&removed).is_empty());
    assert!(launches(&changed).is_empty());
}
