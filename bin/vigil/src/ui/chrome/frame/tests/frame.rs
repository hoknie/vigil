use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::{drawn, quiet};
use crate::ui::chrome::frame::render;
use crate::ui::fixture::screen;
use crate::ui::{Audience, Look, Screen, fixture};

#[test]
fn the_frame_names_the_host_the_open_section_the_facts_and_the_keys() {
    let (page, body) = drawn(fixture::look(), screen("ports"), &fixture::view(), 80, 24);

    assert!(page.contains("app-01"), "{page}");
    assert!(page.contains("alpine"), "{page}");
    assert!(page.contains("What is listening"), "{page}");
    assert!(page.contains("collector(s)"), "{page}");
    assert!(page.contains("3 finding(s)"), "{page}");
    assert!(page.contains("? keys"), "{page}");
    assert!(body.height > 0 && body.width > 0);
}

#[test]
fn the_row_of_screen_names_is_gone_and_the_panel_has_the_band_it_took() {
    let (page, body) = drawn(fixture::look(), screen("ports"), &fixture::view(), 80, 24);

    assert!(
        !page.contains("1 summary"),
        "the numbers are drawn on the main screen now, not along the top: {page}"
    );
    assert_eq!(body.y, 2, "the panel starts one row higher than it did");
    assert_eq!(body.height, 24 - 5);
}

#[test]
fn a_script_gets_the_heading_and_the_facts_without_a_box_or_key_hints() {
    let (page, body) = drawn(
        Look::new(fixture::monochrome(), Audience::Script),
        Screen::SUMMARY,
        &fixture::view(),
        80,
        24,
    );

    assert!(page.contains("app-01"), "{page}");
    assert!(
        page.contains("3 finding(s)"),
        "a file wants the facts too: {page}"
    );
    assert!(!page.contains("q quit"), "a file cannot press keys: {page}");
    assert!(!page.contains('┌'), "and is read with grep: {page}");
    assert_eq!(body.height, 21);
}

#[test]
fn a_terminal_too_short_for_a_frame_gets_the_screen_instead() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 6));

    let body = render(
        fixture::look(),
        Screen::SUMMARY,
        &fixture::view(),
        quiet(),
        buffer.area,
        &mut buffer,
    );

    assert_eq!(body, buffer.area);
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        let (page, _) = drawn(
            fixture::look(),
            Screen::FINDINGS,
            &fixture::view(),
            width,
            24,
        );
        for line in page.lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
    }
}
