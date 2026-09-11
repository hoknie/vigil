use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{height, render};
use crate::ui::helpers::words::text;
use crate::ui::screens::home::{Row, rows};
use crate::ui::{Audience, Look, View, fixture};

fn row(view: &View, name: &str) -> Row {
    rows(view)
        .into_iter()
        .find(|row| row.name == name)
        .unwrap_or_else(|| panic!("no section called {name}"))
}

fn drawn(row: Option<&Row>, look: Look, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 24));
    render(row, look, 0, buffer.area, &mut buffer);
    text::to_text(&buffer)
}

fn squashed(page: &str) -> String {
    page.chars()
        .filter(|letter| !letter.is_whitespace())
        .collect()
}

#[test]
fn the_reason_a_section_is_marked_lives_in_the_panel_and_not_under_the_row() {
    let view = fixture::view_with_trouble();
    let page = drawn(Some(&row(&view, "accounts")), fixture::look(), 60);

    assert!(page.contains("WARNING"), "{page}");
    assert!(
        squashed(&page).contains(&squashed("the owner of one socket could not be resolved")),
        "the agent's own words, whole: {page}"
    );
    assert!(page.contains("degraded"), "{page}");
    assert!(
        page.contains("users"),
        "and which collector said it: {page}"
    );
}

#[test]
fn a_section_with_nothing_to_report_draws_no_heading_over_the_nothing() {
    let page = drawn(Some(&row(&fixture::view(), "ports")), fixture::look(), 60);

    assert!(
        page.contains("PORTS"),
        "its own fields are still there: {page}"
    );
    assert!(page.contains("collector"), "{page}");
    assert!(!page.contains("ABOUT"), "a heading over nothing: {page}");
    assert!(!page.contains("WARNING"), "{page}");
}

#[test]
fn a_reading_this_console_has_no_section_for_says_so_where_the_reader_is_looking() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.push(vigil_model::CollectorStatus {
            name: "resources".into(),
            state: vigil_model::CollectorState::Ok,
            items: 9,
            ..fixture::collector_off()
        });
    }

    let page = drawn(Some(&row(&view, "resources")), fixture::look(), 60);

    assert!(
        squashed(&page).contains(&squashed("has no section for it")),
        "{page}"
    );
    assert!(
        squashed(&page).contains(&squashed("nothing: this console has no section for it")),
        "and that there is no number to press: {page}"
    );
}

#[test]
fn nothing_selected_is_a_sentence_and_not_a_blank_panel() {
    let page = drawn(None, fixture::look(), 60);

    assert!(page.contains("No section is selected"), "{page}");
}

#[test]
fn the_panel_says_the_same_thing_with_no_colour_in_the_palette_at_all() {
    let view = fixture::view_with_trouble();
    let coloured = drawn(Some(&row(&view, "accounts")), fixture::look(), 60);
    let plain = drawn(
        Some(&row(&view, "accounts")),
        Look::new(fixture::monochrome(), Audience::Person),
        60,
    );

    assert_eq!(
        coloured, plain,
        "the reason is carried by characters, so a monochrome terminal loses none of it"
    );
}

#[test]
fn a_panel_is_as_tall_as_what_it_has_to_say_and_no_taller() {
    let view = fixture::view_with_trouble();
    let quiet = height(Some(&row(&fixture::view(), "ports")), fixture::look(), 60);
    let loud = height(Some(&row(&view, "programs")), fixture::look(), 60);

    assert!(quiet > 0);
    assert!(
        loud > quiet,
        "a section with a reason behind it has more to say than one without: {loud} against \
         {quiet}"
    );
    assert_eq!(height(None, fixture::look(), 60), 0);
}

#[test]
fn no_label_in_this_panel_is_cut_in_half_at_any_width_the_console_is_read_at() {
    let view = fixture::view_with_trouble();

    for width in [40u16, 60, 80, 120] {
        for name in ["ports", "accounts", "summary", "findings"] {
            let page = drawn(Some(&row(&view, name)), fixture::look(), width);
            for line in page.lines() {
                assert!(
                    !line.contains('…'),
                    "{width} columns, {name}: a label or a sentence stops in the middle of a \
                     word, and the panel wraps rather than elides: {line}"
                );
                assert!(line.chars().count() <= width as usize, "{line}");
            }
        }
    }
}
