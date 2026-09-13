use serde_json::json;
use vigil_model::Change;

use crate::fixture;
use crate::rules::verdict::processes;

#[test]
fn exactly_one_process_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "a web shell ran `id`",
            Change::Added {
                key: "exec|/bin/sh|www-data".into(),
                after: fixture::program("/bin/sh", "www-data", 33, &["/usr/sbin/php-fpm"]),
            },
            "unexpected_parent",
            "process.unexpected_parent",
        ),
        (
            "something unpacked into /dev/shm and started it",
            Change::Added {
                key: "exec|/dev/shm/payload|www-data".into(),
                after: fixture::program("/dev/shm/payload", "www-data", 33, &[]),
            },
            "process_from_writable_path",
            "process.from_writable_path",
        ),
        (
            "a package upgrade replaced a running binary",
            Change::Changed {
                key: "exec|/usr/sbin/nginx|root".into(),
                before: fixture::program("/usr/sbin/nginx", "root", 0, &[]),
                after: fixture::program_with_deleted_binary("/usr/sbin/nginx", "root", 0),
            },
            "process_binary_deleted",
            "process.binary_deleted",
        ),
        (
            "a program ran as root for the first time",
            Change::Added {
                key: "exec|/usr/bin/tcpdump|root".into(),
                after: fixture::program("/usr/bin/tcpdump", "root", 0, &[]),
            },
            "new_root_process",
            "process.root.new",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = processes(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn a_shell_in_a_writable_directory_under_the_web_server_is_still_one_finding() {
    let change = Change::Added {
        key: "exec|/tmp/.x/sh|www-data".into(),
        after: fixture::program("/tmp/.x/sh", "www-data", 33, &["/usr/sbin/php-fpm"]),
    };

    let fired = processes(&change);

    assert_eq!(fired.len(), 1, "fired: {fired:?}");
    assert_eq!(fired[0].0, "unexpected_parent");
}

#[test]
fn a_program_that_stopped_running_is_reported_by_nobody() {
    let change = Change::Removed {
        key: "exec|/usr/bin/tcpdump|root".into(),
        before: fixture::program("/usr/bin/tcpdump", "root", 0, &[]),
    };

    assert!(processes(&change).is_empty());
}

#[test]
fn the_row_saying_some_executables_could_not_be_read_reaches_no_rule_at_all() {
    let change = Change::Added {
        key: "processes|unresolved".into(),
        after: json!({"exe_resolved": false, "reason": "not permitted"}),
    };

    assert!(processes(&change).is_empty());
}
