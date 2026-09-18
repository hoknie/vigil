use crate::collector::edit::{Edit, add, remove, watching};

const SHIPPED: &str = "\
state_dir: /var/lib/vigil
socket_path: /run/vigil/vigil.sock
retention_days: 90

collectors:
  - network
  - users

schedule:
  network: 30
  users: 300

suppressions: []

reporters: []
";

fn changed(edit: Edit) -> String {
    match edit {
        Edit::Changed(text) => text,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_collector_added_leaves_every_other_byte_of_the_file_where_it_was() {
    let after = changed(add(SHIPPED, "firewall", 60));

    assert!(after.contains("  - firewall\n"), "{after}");
    assert!(after.contains("  firewall: 60\n"), "{after}");
    for kept in [
        "state_dir: /var/lib/vigil",
        "socket_path: /run/vigil/vigil.sock",
        "retention_days: 90",
        "  - network",
        "  - users",
        "  network: 30",
        "  users: 300",
        "suppressions: []",
        "reporters: []",
    ] {
        assert!(after.contains(kept), "{kept} was lost:\n{after}");
    }
    assert_eq!(
        after.lines().count(),
        SHIPPED.lines().count() + 2,
        "exactly two lines were added:\n{after}"
    );
}

#[test]
fn adding_a_collector_that_is_already_watched_changes_nothing_at_all() {
    assert_eq!(add(SHIPPED, "network", 30), Edit::AlreadySo);
    let once = changed(add(SHIPPED, "firewall", 60));
    assert_eq!(add(&once, "firewall", 60), Edit::AlreadySo);
}

#[test]
fn a_disabled_collector_leaves_no_period_behind_because_the_daemon_refuses_one() {
    let with_it = changed(add(SHIPPED, "firewall", 60));

    let after = changed(remove(&with_it, "firewall", &[]));

    assert!(!after.contains("firewall"), "{after}");
    assert_eq!(
        after, SHIPPED,
        "what enable put in, disable takes out, and the file is the one it started as"
    );
}

#[test]
fn removing_one_that_is_not_there_changes_nothing() {
    assert_eq!(remove(SHIPPED, "firewall", &[]), Edit::AlreadySo);
}

#[test]
fn a_file_that_names_no_collectors_means_all_of_them_and_saying_so_is_not_an_edit() {
    let silent = "state_dir: /var/lib/vigil\nreporters: []\n";

    assert_eq!(add(silent, "firewall", 60), Edit::AlreadySo);
}

#[test]
fn taking_one_out_of_a_file_that_named_none_writes_the_rest_by_name() {
    let silent = "state_dir: /var/lib/vigil\nreporters: []\n";
    let known = [("network", 30u32), ("firewall", 60)];

    let after = changed(remove(silent, "firewall", &known));

    assert!(after.contains("  - network"), "{after}");
    assert!(!after.contains("firewall"), "{after}");
    assert!(after.contains("state_dir: /var/lib/vigil"), "{after}");
}

#[test]
fn a_shape_this_command_did_not_write_is_left_alone_and_named() {
    let inline = "collectors: [network, users]\nschedule: {network: 30}\n";

    match add(inline, "firewall", 60) {
        Edit::NotOurs(said) => assert!(said.contains("by hand"), "{said}"),
        other => panic!("a file written another way must not be rewritten: {other:?}"),
    }
}

#[test]
fn a_file_with_no_schedule_block_gets_one_rather_than_a_period_with_nowhere_to_go() {
    let no_schedule = "collectors:\n  - network\n";

    let after = changed(add(no_schedule, "firewall", 60));

    assert!(after.contains("schedule:\n"), "{after}");
    assert!(after.contains("  firewall: 60"), "{after}");
}

#[test]
fn what_the_file_says_now_is_read_before_anything_is_written() {
    assert_eq!(watching(SHIPPED, "network"), Some(true));
    assert_eq!(watching(SHIPPED, "firewall"), Some(false));
    assert_eq!(
        watching("state_dir: /x\n", "firewall"),
        None,
        "a file that names no collectors is watching every one of them, which is not the \
         same answer as naming this one"
    );
}
