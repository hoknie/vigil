use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::harness::{drawn_at, on, opened, press};
use crate::ui::app::App;
use crate::ui::helpers::words::text;
use crate::ui::{Audience, Screen, fixture};

const DEGRADED: &str = "the owner of one socket could not be resolved";

fn troubled(width: u16, height: u16) -> String {
    let mut app = on(
        &opened(&["ui", "--socket", "/nonexistent/vigil.sock"]),
        Screen::Home,
    );
    app.view = fixture::view_with_trouble();
    drawn_at(&app, width, height);
    app.settle();
    drawn_at(&app, width, height)
}

fn troubled_on_the_sick_row(width: u16, height: u16) -> String {
    with_the_panel(width, height, false)
}

fn with_the_panel(width: u16, height: u16, closed: bool) -> String {
    let mut app = on(
        &opened(&["ui", "--socket", "/nonexistent/vigil.sock"]),
        Screen::Home,
    );
    app.view = fixture::view_with_trouble();
    drawn_at(&app, width, height);
    app.settle();
    press(&mut app, KeyCode::Down);
    if !closed {
        press(&mut app, KeyCode::Char('d'));
    }
    drawn_at(&app, width, height)
}

fn captured(width: u16) -> String {
    let mut app = App::new(
        &opened(&["capture", "--socket", "/nonexistent/vigil.sock"]),
        opened(&["capture", "--socket", "/nonexistent/vigil.sock"]).opening(Screen::Home),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view_with_trouble();
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 400));
    app.draw(buffer.area, &mut buffer);
    app.settle();
    text::to_text(&buffer)
}

fn squashed(page: &str) -> String {
    page.chars()
        .filter(|letter| !letter.is_whitespace())
        .collect()
}

#[test]
fn the_panel_is_shut_until_the_key_is_pressed_and_the_same_key_shuts_it_again() {
    let closed = with_the_panel(140, 30, true);
    let open = with_the_panel(140, 30, false);

    assert!(
        !closed.contains("THE SELECTED SECTION"),
        "the panel takes half the width, and a reader who did not ask for it wants the table: \
         {closed}"
    );
    assert!(
        closed.contains("d details"),
        "and the key is offered: {closed}"
    );
    assert!(open.contains("THE SELECTED SECTION"), "{open}");
    assert!(open.contains("d close"), "{open}");
}

#[test]
fn a_section_in_trouble_is_marked_with_the_cursor_on_it_and_with_the_cursor_away() {
    let away = troubled(80, 30);
    let onto_it = troubled_on_the_sick_row(80, 30);

    let marked = |page: &str| {
        page.lines()
            .filter(|line| line.contains("accounts") && line.contains('!'))
            .count()
    };

    assert_eq!(marked(&away), 1, "{away}");
    assert_eq!(
        marked(&onto_it),
        1,
        "the mark went out the moment the reader moved onto the row it is about, which is \
         exactly when it is being read: {onto_it}"
    );
    assert!(
        onto_it
            .lines()
            .any(|line| line.contains("accounts") && line.contains(" > ")),
        "and the cursor is on it too: {onto_it}"
    );
}

#[test]
fn a_monochrome_page_marks_the_same_sections_as_a_coloured_one() {
    let mut app = on(
        &opened(&["ui", "--socket", "/nonexistent/vigil.sock"]),
        Screen::Home,
    );
    app.view = fixture::view_with_trouble();
    let plain = drawn_at(&app, 80, 30);

    let mut coloured = App::new(
        &opened(&["ui", "--socket", "/nonexistent/vigil.sock"]),
        opened(&["ui", "--socket", "/nonexistent/vigil.sock"]).opening(Screen::Home),
        fixture::look().palette,
        Audience::Person,
    );
    coloured.view = fixture::view_with_trouble();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    coloured.draw(buffer.area, &mut buffer);
    let with_colour = text::to_text(&buffer);

    assert_eq!(
        plain, with_colour,
        "the mark is a character before it is a colour, so the two pages read the same"
    );
    assert!(plain.contains('!'), "{plain}");
}

#[test]
fn the_reason_a_section_is_marked_is_on_the_page_at_eighty_columns_too() {
    let page = troubled_on_the_sick_row(80, 30);

    assert!(
        squashed(&page).contains(&squashed(DEGRADED)),
        "there is no room for a panel beside the list at eighty columns, and losing the \
         reason there is losing it for every reader on a narrow terminal: {page}"
    );
    assert!(
        page.contains("THE SELECTED SECTION"),
        "and it is under the caption that says what it is: {page}"
    );
}

#[test]
fn a_wide_terminal_puts_the_reason_beside_the_list_rather_than_under_it() {
    let page = troubled_on_the_sick_row(140, 30);

    assert!(page.contains("SECTIONS"), "{page}");
    assert!(page.contains("THE SELECTED SECTION"), "{page}");
    assert!(
        page.contains(" │ "),
        "the two sit either side of a rule: {page}"
    );
    assert!(squashed(&page).contains(&squashed(DEGRADED)), "{page}");
}

#[test]
fn nothing_a_section_says_about_itself_is_missing_from_what_a_script_is_given() {
    let page = captured(80);

    for said in [
        DEGRADED,
        "auditd is not running on this host",
        "no line of the configuration on this host asks for this reading",
    ] {
        assert!(
            squashed(&page).contains(&squashed(said)),
            "a script cannot move a cursor, so every reason is printed for it: {said:?} is \
             not in\n{page}"
        );
    }
}

#[test]
fn the_row_of_a_section_carries_no_sentence_under_it_any_more() {
    let page = troubled_on_the_sick_row(140, 30);
    let rows: Vec<&str> = page
        .lines()
        .skip_while(|line| !line.contains("SECTION "))
        .take_while(|line| !line.trim().is_empty())
        .collect();

    for line in &rows {
        let list = line.split(" │ ").next().unwrap_or(line);
        assert!(
            !squashed(list).contains(&squashed("could not be resolved")),
            "a table with prose wrapped between its rows cannot be read down a column: {list}"
        );
    }
    assert!(!rows.is_empty(), "{page}");
}

#[test]
fn nothing_this_console_prints_as_a_sentence_is_cut_off_at_any_width_it_is_read_at() {
    for width in [80u16, 100, 106, 140, 200] {
        for closed in [true, false] {
            let page = with_the_panel(width, 40, closed);
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
                assert!(
                    !line.contains('…'),
                    "{width} columns: a word is cut in half and the reader cannot tell what \
                     was there. A cell of a table may be elided; a label, a sentence or a \
                     hint may not, and none of the three is drawn here with room to spare: \
                     {line}"
                );
            }
        }
    }
}

#[test]
fn what_a_script_is_given_carries_every_reason_whether_a_panel_was_asked_for_or_not() {
    let page = captured(80);

    assert!(
        !page.contains("press "),
        "a script cannot press anything: {page}"
    );
    for said in [DEGRADED, "auditd is not running on this host"] {
        assert!(squashed(&page).contains(&squashed(said)), "{page}");
    }
}

#[test]
fn the_summary_a_script_is_given_carries_every_reason_without_a_key_being_pressed() {
    let mut app = App::new(
        &opened(&["capture", "--socket", "/nonexistent/vigil.sock"]),
        opened(&["capture", "--socket", "/nonexistent/vigil.sock"]).opening(Screen::Summary),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view_with_trouble();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 400));
    app.draw(buffer.area, &mut buffer);
    let page = text::to_text(&buffer);

    assert!(squashed(&page).contains(&squashed(DEGRADED)), "{page}");
    assert!(
        !page.contains("press "),
        "a script cannot press anything: {page}"
    );
}

#[test]
fn the_same_key_unfolds_what_a_row_says_on_both_tables_that_have_rows_with_something_to_say() {
    let mut app = on(
        &opened(&["ui", "--socket", "/nonexistent/vigil.sock"]),
        Screen::Summary,
    );
    app.view = fixture::view_with_trouble();
    drawn_at(&app, 80, 60);
    app.settle();

    let folded = drawn_at(&app, 80, 60);
    press(&mut app, KeyCode::Char('d'));
    let unfolded = drawn_at(&app, 80, 60);

    assert!(!squashed(&folded).contains(&squashed(DEGRADED)), "{folded}");
    assert!(
        squashed(&unfolded).contains(&squashed(DEGRADED)),
        "{unfolded}"
    );
    assert!(unfolded.contains("d hide"), "{unfolded}");
    assert!(folded.contains("d why"), "{folded}");
}
