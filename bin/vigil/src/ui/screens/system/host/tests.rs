use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::{CollectorRefusal, CollectorState};

use super::render;
use crate::ui::helpers::words::text;
use crate::ui::screens::system::Showing;
use crate::ui::{Arrows, Reading, Refusal, Search, Sorting, System, View, fixture};

fn showing(search: &Search) -> Showing<'_> {
    Showing {
        showing: System::Host,
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
fn the_reading_says_what_the_host_is_made_of_and_how_much_room_is_left_on_it() {
    let page = drawn(&fixture::view(), 80);

    assert!(page.contains("this host's boot"), "{page}");
    assert!(page.contains("memory and swap"), "{page}");
    assert!(page.contains("/var"), "{page}");
    assert!(page.contains("5%"), "{page}");
    assert!(
        page.contains("least room on /var"),
        "the filesystem nearest to full is the one a reader came for: {page}"
    );
}

#[test]
fn the_table_starts_at_the_top_and_carries_no_banner_over_it() {
    let page = drawn(&fixture::view(), 80);
    let heading = page
        .lines()
        .position(|line| line.contains("KIND") && line.contains("FREE"))
        .expect("a heading");

    assert!(
        heading <= 3,
        "a banner over the table pushes the rows off a short terminal, and what it said \
         belongs on the row or in the panel: {page}"
    );
}

#[test]
fn how_full_a_filesystem_is_is_drawn_in_the_steps_the_agent_reads_it_in() {
    let page = drawn(&fixture::view(), 120);

    assert!(page.contains("60%"), "{page}");
    assert!(
        !page.contains("60.0%"),
        "a number to the byte would change on every reading: {page}"
    );
}

#[test]
fn a_reading_that_was_refused_is_a_screen_that_says_so_and_not_an_empty_table() {
    let mut view = fixture::view();
    view.readings.put(
        "resources",
        Reading::Refused(Refusal::told(CollectorRefusal::new(
            CollectorState::Unavailable,
            "/proc/meminfo could not be read on this host",
        ))),
    );

    let page = drawn(&view, 80);

    assert!(page.contains("refused"), "{page}");
    assert!(page.contains("/proc/meminfo"), "{page}");
    assert!(!page.contains("KIND"), "no empty table under it: {page}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        for line in drawn(&fixture::view(), width).lines() {
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
    }
}
