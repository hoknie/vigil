use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::{Given, busy, drawn, drawn_with, running, showing};
use crate::ui::helpers::words::text;
use crate::ui::screens::ports::{Arrangement, Showing, render};
use crate::ui::{Arrows, Audience, Look, Protocols, fixture};

#[test]
fn the_selected_row_is_marked_with_a_character_and_not_only_a_highlight() {
    let page = drawn(&fixture::view(), 0, 80);

    assert!(
        page.lines().any(|line| line.starts_with(" > ")),
        "a highlight alone is invisible on a monochrome terminal: {page}"
    );
}

#[test]
fn a_files_page_puts_the_tally_under_the_table_rather_than_at_the_foot_of_a_tall_page() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 400));
    let given = Given::default();
    render(
        &fixture::view(),
        Look::new(fixture::monochrome(), Audience::Script),
        &Showing {
            arrows: Arrows::Away,
            ..given.showing(0)
        },
        buffer.area,
        &mut buffer,
    );

    let page = text::to_text(&buffer);
    let lines: Vec<&str> = page.lines().collect();
    assert!(lines.len() < 22, "{} lines of page: {page}", lines.len());
    assert!(
        lines
            .last()
            .expect("a page")
            .contains("listening socket(s)"),
        "{page}"
    );
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        for arrangement in [Arrangement::Flat, Arrangement::ByProgram] {
            for line in drawn_with(
                &busy(),
                &showing(Protocols::default(), arrangement),
                0,
                width,
            )
            .lines()
            {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns, {arrangement:?}: {line}"
                );
            }
        }
    }
}

#[test]
fn the_room_a_command_line_needs_is_not_spent_on_a_program_name_that_is_one_word() {
    let command = "/usr/bin/python3 -m gunicorn --bind 0.0.0.0:8000 app:server";
    let view = running(command);

    for width in [120u16, 160, 200] {
        let page = drawn(&view, 0, width);
        let heading = page
            .lines()
            .find(|line| line.contains("PROGRAM"))
            .expect("a heading");
        let program = heading.find("PROGRAM").expect("the program column");
        let at = heading.find("COMMAND").expect("the command column");

        assert!(
            at - program <= width as usize - at,
            "{width} columns: PROGRAM holds a basename and COMMAND a whole command line, and \
             the wider of the two is the one that is only ever one word: {page}"
        );
    }

    for width in [160u16, 200] {
        let page = drawn(&view, 0, width);
        assert!(
            page.contains(command),
            "{width} columns: the command line is cut while the program name beside it is \
             five letters in a field of thirty: {page}"
        );
    }
}
