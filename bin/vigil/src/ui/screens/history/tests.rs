use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_view::{Pane, Piece, RowKey};

use super::render;
use super::render::CAPTION;
use crate::ui::helpers::words::text;
use crate::ui::{History, Motion, fixture};

const NC: &str = "run|alice|/usr/bin/nc.openbsd";

fn opened(row: &str) -> History {
    let reading = vigil_launches::fixture::launches();
    History::open(vigil_launches::LaunchesPane.history(&reading, &RowKey::of(row)))
}

fn drawn(history: &History, width: u16, height: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    render(history, fixture::look(), buffer.area, &mut buffer);
    text::to_text(&buffer)
}

#[test]
fn the_panel_puts_the_way_back_on_top_and_the_newest_run_above_the_older_one() {
    let page = drawn(&opened(NC), 80, 30);
    let lines: Vec<&str> = page.lines().collect();

    assert!(lines[0].contains("[ \u{2190} Back ]"), "{page}");
    assert!(lines[0].contains(CAPTION), "{page}");
    let newest = lines
        .iter()
        .position(|line| line.contains("1757419207.000:3425"))
        .expect("the second run is drawn");
    let oldest = lines
        .iter()
        .position(|line| line.contains("1757419203.412:3421"))
        .expect("the first run is drawn");
    assert!(
        newest < oldest,
        "a history is read from the top, and the top is the run a reader is asking about: {page}"
    );
}

#[test]
fn the_panel_fits_eighty_and_forty_columns_and_never_runs_off_the_side() {
    for width in [40u16, 80] {
        let page = drawn(&opened(NC), width, 40);
        for line in page.lines() {
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
        assert!(page.contains("3425"), "{width}: the audit id wraps: {page}");
    }
}

#[test]
fn a_row_about_the_reading_opens_a_panel_that_says_why_it_has_no_runs() {
    let page = drawn(&opened("launches|capped"), 80, 20);

    assert!(page.contains("nothing ran under it"), "{page}");
}

#[test]
fn a_panel_scrolled_down_shows_the_later_lines_and_no_longer_the_first() {
    let mut history = History::open(
        (0..40)
            .map(|at| Piece::text(format!("line number {at:02}")))
            .collect(),
    );
    history.scroll(Motion::Last, 8, 40);

    let page = drawn(&history, 80, 10);

    assert!(page.contains("line number 39"), "{page}");
    assert!(!page.contains("line number 00"), "{page}");
    assert!(
        page.contains("[ \u{2190} Back ]"),
        "the way back stays on top however far the panel is scrolled: {page}"
    );
}
