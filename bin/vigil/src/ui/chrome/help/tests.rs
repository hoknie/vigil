use ratatui::widgets::Paragraph as Under;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use super::render::render;
use super::rows::rows;
use crate::ui::fixture;
use crate::ui::helpers::words::text;

#[test]
fn every_key_the_console_uses_is_in_here() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));

    render(fixture::look(), buffer.area, &mut buffer);

    let page = text::to_text(&buffer);
    for key in [
        "1 - 9",
        "PgUp",
        "Enter",
        "o",
        "/",
        "s / f",
        "shift",
        "d silence them",
        "Esc",
        "r",
        "as a tree",
        "a section's words",
        "x / M",
        "K / U / S",
        "H / P / r",
    ] {
        assert!(page.contains(key), "{key} is not on the list: {page}");
    }
    assert!(
        page.contains("the path"),
        "the key that draws the path of a packet works on one list only, and is looked up \
         here or not found at all: {page}"
    );
    assert!(
        page.contains("the runs of a launch"),
        "the key that opens a history works on one list only, and is looked up here or not \
         found at all: {page}"
    );
    assert!(
        !page.contains("Tab"),
        "a key the console ignores must not be offered: {page}"
    );
    assert!(
        page.contains("the numbers are on the main screen"),
        "a number is read off the main screen, not remembered: {page}"
    );
    assert!(page.contains("comes back to it"), "{page}");
    assert!(page.contains("any key closes this"), "{page}");
    assert!(
        page.contains("close or stop it"),
        "the keys on this console that change the host have to say what they take, \
         where the keys are listed: {page}"
    );
    assert!(
        page.contains("units, timers, cron"),
        "the key that stops a service and hides a cron line is looked up here or not \
         found at all: {page}"
    );
}

#[test]
fn the_two_keys_that_go_back_are_one_line_because_they_mean_the_same_thing() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));

    render(fixture::look(), buffer.area, &mut buffer);

    let page = text::to_text(&buffer);
    assert!(page.contains("← or Esc"), "{page}");
    assert!(
        page.contains("a search, the detail, the panel, a rung"),
        "the order of what one press undoes is the thing to look up: {page}"
    );
    assert!(page.contains("the main screen"), "{page}");
    assert!(
        !page.contains("closes the panel"),
        "the two keys were told apart and are not any more: {page}"
    );
}

#[test]
fn what_was_underneath_does_not_show_through_it() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
    Under::new(vec!["xxxxxxxxxxxx".into(); 24]).render(buffer.area, &mut buffer);

    render(fixture::look(), buffer.area, &mut buffer);

    let page = text::to_text(&buffer);
    let middle = page.lines().nth(12).expect("a line through the panel");
    let inside = middle
        .split('│')
        .nth(1)
        .expect("the panel has two sides to it");
    assert!(
        !inside.contains("xxx"),
        "the page underneath is showing through: {middle}"
    );
}

#[test]
fn the_whole_list_fits_the_smallest_terminal_this_console_supports() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));

    render(fixture::look(), buffer.area, &mut buffer);

    let page = text::to_text(&buffer);
    assert!(page.contains("? / q"), "the last group is cut off: {page}");
    assert!(page.contains("startup:"), "{page}");
    assert!(
        page.contains("f: its person or its program"),
        "the launches narrow by the row under the cursor, and a key that does that only \
         on one list is looked up here or not found at all: {page}"
    );
    assert!(page.contains("system: the host"), "{page}");
}

#[test]
fn every_line_of_it_is_written_out_in_full_on_the_smallest_terminal_supported() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));

    render(fixture::look(), buffer.area, &mut buffer);

    let page = text::to_text(&buffer);
    for (keys, meaning) in rows() {
        assert!(
            page.contains(meaning),
            "{keys}: the line ends in the frame and the rest of it is not written anywhere \
             else, so what it says is lost: {page}"
        );
    }
    assert_eq!(
        page.lines()
            .filter(|line| line.contains('\u{2502}'))
            .count(),
        rows().len(),
        "every line of the list is on the screen and none of it is below the bottom of it: \
         {page}"
    );
    assert!(
        page.lines().count() <= 24,
        "and the frame itself closes on the screen: {page}"
    );
}

#[test]
fn the_page_of_keys_says_the_mouse_can_be_put_away_and_how_to_select_text_over_it() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));

    render(fixture::look(), buffer.area, &mut buffer);

    let page = text::to_text(&buffer);
    assert!(
        page.contains("mouse off / on"),
        "a reader whose terminal cannot select text while the console holds the mouse looks \
         the way out up here: {page}"
    );
    assert!(
        page.contains("wheel scrolls"),
        "what the mouse does is worth one line, and the wheel is half of it: {page}"
    );
    assert!(
        page.contains("Shift (Option)"),
        "Terminal.app has no key of its own that gets past the capture, so the one that \
         works is written down: {page}"
    );
}

#[test]
fn it_fits_the_terminal_it_is_opened_on() {
    for (width, height) in [(80u16, 24u16), (120, 40), (40, 12)] {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        render(fixture::look(), buffer.area, &mut buffer);

        for line in text::to_text(&buffer).lines() {
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
    }
}
