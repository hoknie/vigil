use vigil_model::Change;

use crate::rules::fixture;
use crate::rules::verdict::{resources, resources_tick};

const ONE_BOOT: &str = "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31";

const ANOTHER_BOOT: &str = "7d3b9a10-2e45-4c81-b6f7-9a0c1d2e3f40";

#[test]
fn exactly_one_resource_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "the host was restarted between two readings",
            Change::Changed {
                key: "boot|current".into(),
                before: fixture::boot(ONE_BOOT, 1_757_419_200),
                after: fixture::boot(ANOTHER_BOOT, 1_757_720_000),
            },
            "host_rebooted",
            "resource.reboot",
        ),
        (
            "somebody set the clock back an hour under the same kernel",
            Change::Changed {
                key: "boot|current".into(),
                before: fixture::boot(ONE_BOOT, 1_757_419_200),
                after: fixture::boot(ONE_BOOT, 1_757_415_600),
            },
            "clock_stepped",
            "resource.clock_skew",
        ),
        (
            "a filesystem fell under the limit its file names",
            Change::Changed {
                key: "fs|/var".into(),
                before: fixture::filesystem("/var", Some(20), Some(85)),
                after: fixture::filesystem("/var", Some(5), Some(85)),
            },
            "disk_low",
            "resource.disk_low",
        ),
        (
            "a filesystem ran out of inodes while it still reported room",
            Change::Changed {
                key: "fs|/var".into(),
                before: fixture::filesystem("/var", Some(60), Some(20)),
                after: fixture::filesystem("/var", Some(60), Some(0)),
            },
            "inodes_low",
            "resource.inode_low",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = resources(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn a_filesystem_short_of_both_room_and_inodes_is_two_findings_and_not_one() {
    let change = Change::Changed {
        key: "fs|/var".into(),
        before: fixture::filesystem("/var", Some(60), Some(60)),
        after: fixture::filesystem("/var", Some(0), Some(0)),
    };

    let both = resources_tick(std::slice::from_ref(&change));

    assert_eq!(both.len(), 2, "{both:?}");
    assert!(both.contains(&("disk_low".to_string(), "resource.disk_low".to_string())));
    assert!(both.contains(&("inodes_low".to_string(), "resource.inode_low".to_string())));
}

#[test]
fn the_first_reading_of_this_collector_says_nothing_about_a_host_that_is_not_short_of_anything() {
    for change in [
        Change::Added {
            key: "boot|current".into(),
            after: fixture::boot(ONE_BOOT, 1_757_419_200),
        },
        Change::Added {
            key: "memory|summary".into(),
            after: fixture::memory(8_232_091_648, Some(1_073_737_728)),
        },
        Change::Added {
            key: "fs|/var".into(),
            after: fixture::filesystem("/var", Some(60), Some(85)),
        },
    ] {
        assert!(
            resources(&change).is_empty(),
            "a host that has never been read before is not news: {change:?}"
        );
    }
}

#[test]
fn the_row_saying_how_big_this_host_is_reaches_no_rule_at_all() {
    for change in [
        Change::Changed {
            key: "memory|summary".into(),
            before: fixture::memory(8_232_091_648, Some(1_073_737_728)),
            after: fixture::memory(16_464_183_296, None),
        },
        Change::Removed {
            key: "memory|summary".into(),
            before: fixture::memory(8_232_091_648, None),
        },
    ] {
        assert!(
            resources(&change).is_empty(),
            "swap turned off and memory added are worth seeing beside a finding and are not \
             findings of their own: this vocabulary has no kind for either, and inventing one \
             here would be a change to a published contract made in a rule: {change:?}"
        );
    }
}

#[test]
fn a_filesystem_that_was_unmounted_is_not_a_filesystem_that_ran_out_of_room() {
    let change = Change::Removed {
        key: "fs|/mnt/scratch".into(),
        before: fixture::filesystem("/mnt/scratch", Some(0), Some(0)),
    };

    assert!(resources(&change).is_empty());
}
