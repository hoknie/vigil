use vigil_model::Change;

use crate::fixture;
use crate::rules::verdict::files;
use vigil_rules::fixture as neighbours;

#[test]
fn exactly_one_file_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "somebody edited a watched file",
            Change::Changed {
                key: "file|/etc/ssh/sshd_config".into(),
                before: fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                after: fixture::watched_file("/etc/ssh/sshd_config", "0600", "b4"),
            },
            "file_changed",
            "file.changed",
        ),
        (
            "somebody opened a watched file to everybody",
            Change::Changed {
                key: "file|/etc/ssh/sshd_config".into(),
                before: fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                after: fixture::watched_file("/etc/ssh/sshd_config", "0666", "a9"),
            },
            "file_permissions_changed",
            "file.permissions_changed",
        ),
        (
            "a watched program became one that runs as its owner",
            Change::Changed {
                key: "file|/usr/bin/at".into(),
                before: fixture::watched_file("/usr/bin/at", "0755", "a9"),
                after: fixture::watched_file("/usr/bin/at", "4755", "a9"),
            },
            "file_suid_new",
            "file.suid.new",
        ),
        (
            "a directory of the path was opened to everybody",
            Change::Changed {
                key: "directory|/usr/local/bin".into(),
                before: fixture::watched_directory("/usr/local/bin", "0755"),
                after: fixture::watched_directory("/usr/local/bin", "0777"),
            },
            "path_writable_by_all",
            "file.path_writable_by_all",
        ),
        (
            "a watched file is gone",
            Change::Changed {
                key: "file|/etc/ssh/sshd_config".into(),
                before: fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                after: fixture::watched_file_absent("/etc/ssh/sshd_config"),
            },
            "file_changed",
            "file.changed",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = files(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn one_edit_that_changed_the_content_and_the_mode_is_two_facts_and_two_findings() {
    let change = Change::Changed {
        key: "file|/etc/ssh/sshd_config".into(),
        before: fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
        after: fixture::watched_file("/etc/ssh/sshd_config", "0666", "b4"),
    };

    let both = files(&change);

    assert_eq!(both.len(), 2, "{both:?}");
    assert!(both.contains(&("file_changed".to_string(), "file.changed".to_string())));
    assert!(both.contains(&(
        "file_permissions_changed".to_string(),
        "file.permissions_changed".to_string()
    )));
}

#[test]
fn the_first_reading_of_a_host_says_nothing_about_the_files_it_shipped_with() {
    for change in [
        Change::Added {
            key: "file|/etc/ssh/sshd_config".into(),
            after: fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
        },
        Change::Added {
            key: "file|/usr/bin/at".into(),
            after: fixture::watched_file("/usr/bin/at", "4755", "a9"),
        },
        Change::Added {
            key: "directory|/usr/bin".into(),
            after: fixture::watched_directory("/usr/bin", "0755"),
        },
        Change::Added {
            key: "file|/etc/pam.d/sshd".into(),
            after: fixture::watched_file_absent("/etc/pam.d/sshd"),
        },
    ] {
        assert!(
            files(&change).is_empty(),
            "a host that has never been read before is not news: {change:?}"
        );
    }
}

#[test]
fn a_path_that_stopped_being_watched_is_not_a_finding_about_the_file() {
    let change = Change::Removed {
        key: "file|/etc/hosts".into(),
        before: fixture::watched_file("/etc/hosts", "0644", "a9"),
    };

    assert!(
        files(&change).is_empty(),
        "a row that left the reading means the path left the configuration, and somebody \
         editing the configuration is not an incident on this host"
    );
}

#[test]
fn a_row_of_another_collector_reaches_no_rule_of_this_module() {
    for key in ["fs|/var", "tcp|0.0.0.0:443", "container|3ab1c0f2d4e5"] {
        let change = Change::Changed {
            key: key.into(),
            before: neighbours::of_another_collector(key),
            after: neighbours::of_another_collector(key),
        };

        assert!(
            files(&change).is_empty(),
            "these rules are handed every change of their own reading and nothing else; a \
             rule that answers about {key} would report the same host twice, once here and \
             once in the module that owns it"
        );
    }
}
