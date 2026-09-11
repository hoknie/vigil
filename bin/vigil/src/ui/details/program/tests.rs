use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::render;
use crate::ui::helpers::words::text;
use crate::ui::screens::programs::rows;
use crate::ui::{Program, Search, View, fixture};

fn drawn(view: &View, program: Program, key: &str) -> String {
    let search = Search::default();
    let listed = rows(view, program, &search);
    let row = listed
        .iter()
        .find(|row| row.key == key)
        .unwrap_or_else(|| panic!("no row keyed {key}"));

    let mut buffer = Buffer::empty(Rect::new(0, 0, 160, 60));
    render(
        Some(row),
        program,
        fixture::look(),
        0,
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

#[test]
fn a_program_says_its_whole_path_the_account_it_runs_as_and_who_started_it() {
    let page = drawn(
        &fixture::view(),
        Program::Running,
        "exec|/tmp/.x/nc|www-data",
    );

    assert!(page.contains("/tmp/.x/nc"), "{page}");
    assert!(page.contains("www-data"), "{page}");
    assert!(page.contains("/bin/sh"), "{page}");
    assert!(page.contains("unlinked from disk"), "{page}");
    assert!(page.contains("anybody on this host can write to"), "{page}");
}

#[test]
fn a_command_line_that_differs_between_the_processes_says_so_instead_of_showing_one_of_them() {
    let page = drawn(
        &fixture::view(),
        Program::Running,
        "exec|/usr/sbin/nginx|www-data",
    );

    assert!(page.contains("varies between the processes"), "{page}");
}

#[test]
fn a_program_is_suppressed_by_the_key_the_rules_key_it_by_and_not_by_the_snapshot_key() {
    let page = drawn(
        &fixture::view(),
        Program::Running,
        "exec|/usr/sbin/nginx|root",
    );

    assert!(page.contains("suppressions:"), "{page}");
    assert!(
        page.contains("process|exec|/usr/sbin/nginx|root"),
        "a suppression matches a finding_key, and that family adds its own prefix: {page}"
    );
}

#[test]
fn a_launch_says_the_file_was_there_when_the_record_was_read_and_not_that_it_is_there_now() {
    let page = drawn(
        &fixture::view_with_launches(),
        Program::Launches,
        "run|root|/dev/shm/payload",
    );

    assert!(
        page.contains("was not on disk when this record was read"),
        "{page}"
    );
    assert!(page.contains("not what is true now"), "{page}");
}

#[test]
fn a_launch_is_suppressed_by_the_whole_key_because_that_family_adds_no_prefix() {
    let page = drawn(
        &fixture::view_with_launches(),
        Program::Launches,
        "run|alice|/usr/bin/nmap",
    );

    assert!(page.contains("suppressions:"), "{page}");
    assert!(page.contains("\"run|alice|/usr/bin/nmap\""), "{page}");
    assert!(
        !page.contains("launches|run|alice"),
        "a prefix invented here is a suppression that matches nothing: {page}"
    );
}

#[test]
fn arguments_nobody_asked_the_agent_to_keep_are_named_as_not_kept_rather_than_left_blank() {
    let page = drawn(
        &fixture::view_with_launches(),
        Program::Launches,
        "run|alice|/usr/bin/nmap",
    );

    assert!(page.contains("not recorded"), "{page}");
}
