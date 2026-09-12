use serde_json::json;
use vigil_model::Change;

use crate::rules::fixture;
use crate::rules::verdict::persistence;

#[test]
fn exactly_one_persistence_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "a package installed a service",
            Change::Added {
                key: "unit|nginx.service".into(),
                after: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
            },
            "new_unit",
            "persistence.unit.new",
        ),
        (
            "something dropped a unit that starts a binary in /tmp",
            Change::Added {
                key: "unit|update.service".into(),
                after: fixture::unit("update.service", "/tmp/.x/implant", "root"),
            },
            "new_unit",
            "persistence.unit.new",
        ),
        (
            "systemctl edit pointed an existing unit somewhere else",
            Change::Changed {
                key: "unit|nginx.service".into(),
                before: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
                after: fixture::unit("nginx.service", "/tmp/.x/nginx", "root"),
            },
            "unit_command_changed",
            "file.changed",
        ),
        (
            "a timer appeared",
            Change::Added {
                key: "timer|certbot.timer".into(),
                after: fixture::timer("certbot.timer", &["daily"], None),
            },
            "new_timer",
            "persistence.timer.new",
        ),
        (
            "crontab -e added a job",
            Change::Added {
                key: "cron|/var/spool/cron/crontabs/deploy|deploy|/usr/local/bin/sync".into(),
                after: fixture::cron_job(
                    "/var/spool/cron/crontabs/deploy",
                    "deploy",
                    "*/5 * * * *",
                    "/usr/local/bin/sync",
                ),
            },
            "new_cron_job",
            "persistence.cron.new",
        ),
        (
            "echo /lib/hider.so > /etc/ld.so.preload",
            Change::Changed {
                key: "preload|/etc/ld.so.preload".into(),
                before: fixture::preload(&[]),
                after: fixture::preload(&["/lib/hider.so"]),
            },
            "preload_changed",
            "persistence.preload_changed",
        ),
        (
            "a line was appended to /etc/profile.d/00-aliases.sh",
            Change::Changed {
                key: "script|/etc/profile.d/00-aliases.sh".into(),
                before: fixture::script("/etc/profile.d/00-aliases.sh", "profile", "aaa"),
                after: fixture::script("/etc/profile.d/00-aliases.sh", "profile", "bbb"),
            },
            "shell_profile_changed",
            "persistence.shell_profile_changed",
        ),
        (
            "/etc/rc.local grew a line",
            Change::Changed {
                key: "script|/etc/rc.local".into(),
                before: fixture::script("/etc/rc.local", "boot", "aaa"),
                after: fixture::script("/etc/rc.local", "boot", "bbb"),
            },
            "shell_profile_changed",
            "persistence.shell_profile_changed",
        ),
        (
            "insmod",
            Change::Added {
                key: "module|diamorphine".into(),
                after: fixture::kernel_module("diamorphine", 16384),
            },
            "kernel_module_loaded",
            "persistence.kernel_module.loaded",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = persistence(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn things_going_away_in_this_family_are_carried_in_the_snapshot_and_reported_by_nobody() {
    for change in [
        Change::Removed {
            key: "unit|nginx.service".into(),
            before: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
        },
        Change::Removed {
            key: "timer|certbot.timer".into(),
            before: fixture::timer("certbot.timer", &["daily"], None),
        },
        Change::Removed {
            key: "cron|/etc/crontab|root|/usr/bin/backup".into(),
            before: fixture::cron_job("/etc/crontab", "root", "0 3 * * *", "/usr/bin/backup"),
        },
        Change::Removed {
            key: "module|loop".into(),
            before: fixture::kernel_module("loop", 32768),
        },
    ] {
        assert!(persistence(&change).is_empty(), "{change:?}");
    }
}

#[test]
fn the_row_saying_the_kernel_module_list_could_not_be_read_reaches_no_rule_at_all() {
    let change = Change::Added {
        key: "modules|unreadable".into(),
        after: json!({"readable": false, "reason": "/proc/modules could not be read"}),
    };

    assert!(persistence(&change).is_empty());
}
