use std::fs;

use crate::SpoolWriter;
use vigil_collect::{CollectError, Collector, Health};

use super::harness::{LAUNCH, collector, plugin_config_beside, spool_beside, workspace};

#[test]
fn a_host_with_no_auditd_says_so_instead_of_reporting_that_nobody_ran_anything() {
    let path = workspace("absent.log");
    let collector = collector(&path);

    assert!(matches!(collector.available(), Health::Unavailable(_)));
    assert!(matches!(collector.collect(), Err(CollectError::Absent(_)),));
}

#[test]
fn a_log_with_nothing_of_ours_in_it_names_both_hosts_that_look_like_that_and_neither_as_fact() {
    let path = workspace("untagged.log");
    fs::write(
        &path,
        "type=USER_LOGIN msg=audit(1757419203.412:3400): pid=1 uid=0 auid=1000 res=success\n",
    )
    .expect("write");

    match collector(&path).available() {
        Health::Degraded(detail) => {
            assert!(detail.contains("auditctl -l"), "{detail}");
            assert!(detail.contains("augenrules"), "{detail}");
            assert!(
                detail.contains("nobody has run a command") && detail.contains("never loaded"),
                "this agent cannot tell a loaded rule nothing has matched from a rule that \
                 was never loaded, and a text that picks one sends an operator to load a \
                 rule that may already be standing: {detail}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_kernel_saying_it_took_our_rule_is_a_collector_that_is_well_and_not_one_complaining() {
    let path = workspace("rule-taken.log");
    fs::write(
        &path,
        "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"vigil_exec\" list=4 res=1\n",
    )
    .expect("write");
    let collector = collector(&path);

    let before = collector.available();
    let reading = collector.collect().expect("readable");
    let after = collector.available();

    assert!(
        matches!(before, Health::Degraded(_)),
        "nothing has been read yet, so the two hosts still look alike: {before:?}"
    );
    assert!(
        reading.items.keys().all(|key| !key.starts_with("run|")),
        "a rule being taken is not somebody running something"
    );
    assert_eq!(
        after,
        Health::Ok,
        "the rule is standing and nothing has matched it: that is a reading, not a refusal, \
         and the host it happens on is a quiet one rather than a misconfigured one"
    );
}

#[test]
fn a_launch_read_once_keeps_the_collector_well_on_every_quiet_reading_after_it() {
    let path = workspace("quiet-after.log");
    fs::write(&path, LAUNCH).expect("write");
    let collector = collector(&path);

    collector.collect().expect("readable");
    fs::write(&path, "").expect("truncate");

    assert_eq!(
        collector.available(),
        Health::Ok,
        "the tag falls out of the tail of a rotated log on any quiet host, and complaining \
         then would make every quiet host look misconfigured"
    );
}

#[test]
fn a_registered_plugin_that_brings_nothing_is_not_the_same_answer_as_a_missing_rule() {
    let path = workspace("silent-plugin.log");
    fs::write(&path, LAUNCH).expect("write");
    fs::write(
            plugin_config_beside(&path),
            "active = yes\ndirection = out\npath = /usr/sbin/vigil-audit-plugin\ntype = always\nformat = string\n",
        )
        .expect("write");

    match collector(&path).available() {
        Health::Degraded(detail) => {
            assert!(detail.contains("has never run"), "{detail}");
            assert!(detail.contains("restart auditd"), "{detail}");
            assert!(!detail.contains("augenrules"), "{detail}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_plugin_that_has_started_and_delivered_nothing_is_not_told_to_restart_anything() {
    let path = workspace("quiet-plugin.log");
    fs::write(&path, LAUNCH).expect("write");
    fs::write(plugin_config_beside(&path), "active = yes\n").expect("write");
    SpoolWriter::open(spool_beside(&path), 1024 * 1024).expect("opens");

    match collector(&path).available() {
        Health::Degraded(detail) => {
            assert!(detail.contains("delivered nothing"), "{detail}");
            assert!(!detail.contains("has never run"), "{detail}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_host_reading_the_log_with_no_plugin_registered_is_healthy() {
    let path = workspace("fallback.log");
    fs::write(&path, LAUNCH).expect("write");

    assert_eq!(collector(&path).available(), Health::Ok);
}
