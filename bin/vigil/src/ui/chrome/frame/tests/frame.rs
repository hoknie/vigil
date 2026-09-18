use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::{drawn, quiet};
use crate::ui::chrome::frame::Hints;
use crate::ui::chrome::frame::render;
use crate::ui::fixture::screen;
use crate::ui::helpers::words::text;
use crate::ui::{Audience, Look, Screen, fixture};

#[test]
fn the_frame_names_the_host_the_open_section_the_facts_and_the_keys() {
    let (page, body) = drawn(fixture::look(), screen("network"), &fixture::view(), 80, 24);

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
    let (page, body) = drawn(fixture::look(), screen("network"), &fixture::view(), 80, 24);

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
    assert!(
        !page.contains('┌') && !page.contains('\u{256d}') && !page.contains('\u{250f}'),
        "and is read with grep: {page}"
    );
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

#[test]
fn the_frame_of_a_section_is_thick_while_the_arrows_are_its_own_and_a_plain_line_over_panes() {
    let framed = |panes: bool| {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
        let body = render(
            fixture::look(),
            screen("network"),
            &fixture::view(),
            Hints { panes, ..quiet() },
            buffer.area,
            &mut buffer,
        );
        (text::to_text(&buffer), body)
    };

    let (alone, _) = framed(false);
    assert!(
        alone
            .lines()
            .nth(1)
            .is_some_and(|line| line.starts_with("\u{250f} \u{25b8} What is listening")),
        "with nothing framed inside it, the section's own frame is where the arrows are: {alone}"
    );
    assert!(alone.contains('\u{251b}'), "{alone}");

    let (over, body) = framed(true);
    let header = over.lines().nth(1).unwrap_or_default();
    assert!(
        header.starts_with(" What is listening") && header.ends_with("as of 09:00:01"),
        "over panes that have frames of their own, the section is named on a plain line with \
         the time of its reading, so no side of the screen carries two borders: {over}"
    );
    for corner in [
        '\u{256d}', '\u{256f}', '\u{250f}', '\u{251b}', '\u{2502}', '\u{2503}',
    ] {
        assert!(
            !over.contains(corner),
            "the section draws no frame of its own around panes that are framed: {over}"
        );
    }
    assert_eq!(
        body,
        Rect::new(0, 2, 80, 20),
        "the two columns and the row the outer frame took go to the panes"
    );
}
