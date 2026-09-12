use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::{CollectorRefusal, CollectorState, Snapshot};

use super::render;
use crate::ui::helpers::words::text;
use crate::ui::screens::system::Showing;
use crate::ui::{Arrows, Reading, Refusal, Search, Sorting, System, View, fixture};

fn showing(search: &Search) -> Showing<'_> {
    Showing {
        showing: System::Files,
        search,
        cursor: 0,
        elsewhere: 0,
        arrows: Arrows::List,
        gone: None,
        sorting: Sorting::default(),
    }
}

fn drawn(view: &View, width: u16) -> String {
    let search = Search::default();
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 24));
    render(
        view,
        fixture::look(),
        &showing(&search),
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

#[test]
fn every_watched_path_says_its_mode_its_owner_and_how_it_stands() {
    let page = drawn(&fixture::view(), 80);

    assert!(page.contains("/etc/ssh/sshd_config"), "{page}");
    assert!(page.contains("0600"), "{page}");
    assert!(page.contains("0:0"), "{page}");
    assert!(
        page.contains("/usr/bin"),
        "the directories on PATH too: {page}"
    );
}

#[test]
fn a_file_that_is_not_there_reads_as_watched_and_missing_rather_than_as_a_blank_row() {
    let page = drawn(&fixture::view(), 80);
    let row = page
        .lines()
        .find(|line| line.contains("/etc/pam.d/sshd"))
        .expect("the missing file is still a row");

    assert!(row.contains("not there"), "{row}");
}

#[test]
fn a_file_anyone_may_write_to_and_one_that_runs_as_its_owner_are_marked_by_what_they_are() {
    let page = drawn(&fixture::view(), 80);
    let suid = page
        .lines()
        .find(|line| line.contains("/usr/bin/newgrp"))
        .expect("the suid file");
    let writable = page
        .lines()
        .find(|line| line.contains("/usr/local/bin"))
        .expect("the directory");

    assert!(suid.contains("suid"), "{suid}");
    assert!(
        !writable.contains("anyone may write"),
        "0775 is the group, not anyone: {writable}"
    );
}

#[test]
fn a_host_where_nothing_is_named_says_so_in_words_rather_than_drawing_an_empty_table() {
    let mut view = fixture::view();
    view.readings.put(
        "files",
        Reading::Taken(Snapshot::new("files", "2026-09-09T09:00:00.000Z")),
    );

    let page = drawn(&view, 80);

    assert!(page.contains("No path is being watched"), "{page}");
    assert!(page.contains("configuration"), "{page}");
    assert!(!page.contains("PATH   "), "{page}");
}

#[test]
fn a_reading_that_was_refused_is_a_screen_that_says_so_and_not_an_empty_table() {
    let mut view = fixture::view();
    view.readings.put(
        "files",
        Reading::Refused(Refusal::told(CollectorRefusal::new(
            CollectorState::Unavailable,
            "no path is named in the configuration and PATH could not be read",
        ))),
    );

    let page = drawn(&view, 80);

    assert!(page.contains("refused"), "{page}");
    assert!(page.contains("no path is named"), "{page}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        for line in drawn(&fixture::view(), width).lines() {
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
    }
}
