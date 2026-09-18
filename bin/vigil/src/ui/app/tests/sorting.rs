use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::harness::{app, drawn, drawn_at, into, number, opened, press};
use crate::ui::app::App;
use crate::ui::fixture::screen;
use crate::ui::helpers::words::text;
use crate::ui::{Audience, Screen, fixture};

fn chose(app: &mut App, option: usize) {
    press(app, KeyCode::Char('s'));
    while app.chooser.at() != option {
        press(app, KeyCode::Right);
    }
    press(app, KeyCode::Enter);
}

fn at(page: &str, wanted: &str) -> usize {
    page.find(wanted)
        .unwrap_or_else(|| panic!("{wanted} is not on the page: {page}"))
}

#[test]
fn a_column_sorts_the_rows_both_ways_round_and_the_screen_says_which_way() {
    let mut app = app();
    press(&mut app, number(Screen::FINDINGS));

    chose(&mut app, 3);
    let up = drawn_at(&app, 120, 30);
    chose(&mut app, 4);
    let down = drawn_at(&app, 120, 30);

    assert!(
        at(&up, "A user logged in") < at(&up, "A new listening port"),
        "the quiet ones first: {up}"
    );
    assert!(
        at(&down, "A new listening port") < at(&down, "A user logged in"),
        "the loud ones first: {down}"
    );
    assert!(down.contains("sorted by SEVERITY, largest first"), "{down}");
    assert!(up.contains("sorted by SEVERITY, smallest first"), "{up}");
}

#[test]
fn a_sort_belongs_to_the_list_it_was_chosen_in() {
    let mut app = app();
    press(&mut app, number(Screen::FINDINGS));
    chose(&mut app, 4);

    into(&mut app, screen("network"), 120, 30);
    let ports = drawn_at(&app, 120, 30);
    assert!(
        !ports.contains("sorted by"),
        "the order chosen on another list followed the reader here: {ports}"
    );

    press(&mut app, number(Screen::FINDINGS));
    assert!(
        drawn_at(&app, 120, 30).contains("sorted by SEVERITY"),
        "and the list it was chosen in kept it: {}",
        drawn_at(&app, 120, 30)
    );
}

#[test]
fn the_order_a_reader_chose_puts_them_at_the_top_of_the_list_and_not_at_a_row_number() {
    let mut app = app();
    press(&mut app, number(Screen::FINDINGS));
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);

    chose(&mut app, 4);

    assert_eq!(
        app.nav.findings.at(),
        0,
        "a cursor left on row three of an order that no longer exists is a cursor on nothing \
         the reader was looking at"
    );
}

#[test]
fn what_a_script_is_given_is_the_order_the_agent_sent_and_says_nothing_about_an_order() {
    let mut app = App::new(
        &opened(&["capture", "--socket", "/nonexistent/vigil.sock"]),
        opened(&["capture", "--socket", "/nonexistent/vigil.sock"]).opening(Screen::FINDINGS),
        fixture::monochrome(),
        Audience::Script,
    );
    app.view = fixture::view();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 120, 400));
    app.draw(buffer.area, &mut buffer);
    let page = text::to_text(&buffer);

    assert!(
        at(&page, "A new listening port") < at(&page, "A user logged in"),
        "a script reads the rows in the order the agent sent them: {page}"
    );
    assert!(
        !page.contains("sorted by"),
        "nobody pressed anything, so there is no order to announce: {page}"
    );
}

#[test]
fn the_grouped_view_is_an_order_already_and_says_where_the_sorting_is_done() {
    let mut app = app();
    press(&mut app, number(screen("network")));
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Char('s'));

    let page = drawn_at(&app, 120, 30);
    assert!(page.contains("an order already"), "{page}");
    assert!(page.contains("sockets view"), "{page}");
    assert!(!page.contains("sort by"), "no choice was opened: {page}");
}

#[test]
fn a_screen_that_sorts_says_so_on_the_key_line_and_the_choice_says_how_to_walk_it() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);

    assert!(drawn(&app).contains("s sort"), "{}", drawn(&app));

    press(&mut app, KeyCode::Char('s'));
    let choosing = drawn(&app);
    assert!(choosing.contains("sort by"), "{choosing}");
    assert!(choosing.contains("as the agent sends it"), "{choosing}");
    assert!(choosing.contains("Enter apply"), "{choosing}");
}
