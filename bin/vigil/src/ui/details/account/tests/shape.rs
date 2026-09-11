use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::drawn;
use crate::ui::details::account::render;
use crate::ui::helpers::words::text as page;
use crate::ui::screens::accounts;
use crate::ui::{Search, Subject, fixture};

#[test]
fn with_nothing_selected_it_says_how_to_select_something() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 10));
    render(
        None,
        &fixture::view(),
        fixture::look(),
        0,
        buffer.area,
        &mut buffer,
    );

    assert!(page::to_text(&buffer).contains("press →"));
}

#[test]
fn the_detail_answers_the_same_way_whatever_the_search_is_doing() {
    let view = fixture::view();
    let mut search = Search::default();
    search.start();
    for character in "deploy".chars() {
        search.type_character(character);
    }
    search.accept();

    let rows = accounts::rows(&view, Subject::Users, &search);
    let row = rows
        .iter()
        .find(|row| row.key == "account|deploy")
        .expect("still on the list");
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 60));
    render(
        Some(row),
        &view,
        fixture::look(),
        0,
        buffer.area,
        &mut buffer,
    );

    assert!(page::to_text(&buffer).contains("may start a container"));
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [56u16, 80, 120] {
        for line in drawn(&fixture::view(), "account|deploy", width).lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
    }
}
