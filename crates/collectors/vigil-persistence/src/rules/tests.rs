use serde_json::json;
use vigil_model::Change;

use super::verdict::persistence;
use crate::fixture;

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
            "crontab -e took a job out again",
            Change::Removed {
                key: "cron|/var/spool/cron/crontabs/deploy|deploy|/usr/local/bin/sync".into(),
                before: fixture::cron_job(
                    "/var/spool/cron/crontabs/deploy",
                    "deploy",
                    "*/5 * * * *",
                    "/usr/local/bin/sync",
                ),
            },
            "cron_job_removed",
            "persistence.cron.removed",
        ),
        (
            "the same job now runs at every boot",
            Change::Changed {
                key: "cron|/var/spool/cron/crontabs/deploy|deploy|/usr/local/bin/sync".into(),
                before: fixture::cron_job(
                    "/var/spool/cron/crontabs/deploy",
                    "deploy",
                    "*/5 * * * *",
                    "/usr/local/bin/sync",
                ),
                after: fixture::cron_job(
                    "/var/spool/cron/crontabs/deploy",
                    "deploy",
                    "@reboot",
                    "/usr/local/bin/sync",
                ),
            },
            "cron_job_changed",
            "persistence.cron.changed",
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
fn of_the_things_going_away_in_this_family_a_cron_line_is_the_one_that_is_reported() {
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
            key: "module|loop".into(),
            before: fixture::kernel_module("loop", 32768),
        },
    ] {
        assert!(
            persistence(&change).is_empty(),
            "a unit file, a timer file and a module come and go with every package upgrade \
             on a host, and a finding on each is the noise this product dies of: {change:?}"
        );
    }

    let gone = persistence(&Change::Removed {
        key: "cron|/etc/crontab|root|/usr/bin/backup".into(),
        before: fixture::cron_job("/etc/crontab", "root", "0 3 * * *", "/usr/bin/backup"),
    });

    assert_eq!(
        gone.len(),
        1,
        "a line that left a crontab is reported since 2026-09-16, by the owner's decision \
         (docs/designs/2026-09-16-DESIGN-console-units.md): the console can comment one out, \
         so a job that stops being scheduled is now something this agent may have done, and \
         the reading has to say so either way"
    );
    assert_eq!(gone[0].1, "persistence.cron.removed");
}

#[test]
fn a_unit_that_changed_only_in_which_other_units_pull_it_in_raises_nothing() {
    let older = fixture::unit("nginx.service", "/usr/sbin/nginx", "root");
    let mut carried = older.clone();
    carried["pulled_in_by"] = json!([]);
    let mut grown = carried.clone();
    grown["pulled_in_by"] = json!(["multi-user.target"]);

    for (what, before, after) in [
        (
            "the first reading after an upgrade writes the list for every unit",
            older,
            carried.clone(),
        ),
        ("another unit file started wanting this one", carried, grown),
    ] {
        let fired = persistence(&Change::Changed {
            key: "unit|nginx.service".into(),
            before,
            after,
        });
        assert!(
            fired.is_empty(),
            "{what}: this unit's own file did not change, and the other file is its own \
             finding, so this one would be noise: {fired:?}"
        );
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
