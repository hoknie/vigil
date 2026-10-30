use super::harness::{SHIPPED, changed};
use crate::{Edit, Entry, Naming, add, remove_one, remove_written, silenced};

fn tunnel() -> Entry {
    Entry {
        key: "port.listen|udp|*".into(),
        naming: Naming::Pattern,
        program: Some("netbird".into()),
        reason: "vpn mesh changes its ports".into(),
        ..Entry::default()
    }
}

fn new_ports_of(program: &str) -> Entry {
    Entry {
        naming: Naming::Nothing,
        program: Some(program.into()),
        kind: Some("port.listen.new".into()),
        reason: "vpn mesh".into(),
        ..Entry::default()
    }
}

#[test]
fn an_entry_with_a_program_and_a_pattern_is_read_back_as_the_daemon_will_read_it() {
    let after = changed(add(SHIPPED, &[tunnel()]));
    let held = silenced(&after).expect("what was written loads");

    assert_eq!(held.len(), 1, "{after}");
    assert_eq!(held[0].program.as_deref(), Some("netbird"));
    assert_eq!(
        held[0].finding_key_match.as_deref(),
        Some("port.listen|udp|*")
    );
    assert_eq!(
        held[0].describe(),
        "program netbird · port.listen|udp|* — vpn mesh changes its ports"
    );
}

#[test]
fn an_entry_naming_a_program_and_no_key_starts_with_the_program() {
    let after = changed(add(SHIPPED, &[new_ports_of("/usr/bin/netbird")]));

    assert!(
        after.contains("  - program: \"/usr/bin/netbird\"\n    kind: \"port.listen.new\"\n"),
        "{after}"
    );
    let held = silenced(&after).expect("what was written loads");
    assert_eq!(
        held[0].describe(),
        "program /usr/bin/netbird (port.listen.new) — vpn mesh"
    );
}

#[test]
fn the_same_program_written_twice_is_one_entry_and_two_programs_are_two() {
    let once = changed(add(SHIPPED, &[new_ports_of("/usr/bin/netbird")]));

    assert_eq!(
        add(&once, &[new_ports_of("/usr/bin/netbird")]),
        Edit::AlreadySo
    );
    assert!(matches!(
        add(&once, &[new_ports_of("/usr/sbin/wg")]),
        Edit::Changed { entries: 1, .. }
    ));
    assert!(
        matches!(add(&once, &[tunnel()]), Edit::Changed { entries: 1, .. }),
        "a program narrowed by a pattern is a statement of its own"
    );
}

#[test]
fn taking_out_a_program_and_its_pattern_leaves_the_same_program_narrowed_by_kind() {
    let both = changed(add(SHIPPED, &[tunnel(), new_ports_of("netbird")]));

    let after = changed(remove_written(
        &both,
        &[],
        Some("netbird"),
        Some("port.listen|udp|*"),
    ));

    let held = silenced(&after).expect("what is left loads");
    assert_eq!(held.len(), 1, "{after}");
    assert_eq!(held[0].kind.as_deref(), Some("port.listen.new"));
}

#[test]
fn an_entry_taken_out_from_the_screen_is_the_one_with_its_program_and_not_its_neighbour() {
    let both = changed(add(
        SHIPPED,
        &[
            new_ports_of("/usr/bin/netbird"),
            new_ports_of("/usr/sbin/wg"),
        ],
    ));
    let held = silenced(&both).expect("loads");

    let after = changed(remove_one(&both, &held[0]));

    let left = silenced(&after).expect("loads");
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].program.as_deref(), Some("/usr/sbin/wg"));
}
