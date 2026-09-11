use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::drawn_at;
use crate::ui::helpers::words::text as page;
use crate::ui::screens::accounts::{Showing, render};
use crate::ui::{Arrows, Audience, Look, Search, Subject, fixture};

#[test]
fn a_files_page_puts_the_tally_under_the_table() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 400));
    render(
        &fixture::view(),
        Look::new(fixture::monochrome(), Audience::Script),
        &Showing {
            subject: Subject::Users,
            search: &Search::default(),
            cursor: 0,
            elsewhere: 0,
            arrows: Arrows::Away,
        },
        buffer.area,
        &mut buffer,
    );

    let page = page::to_text(&buffer);
    assert!(page.lines().count() < 20, "{page}");
    assert!(
        page.lines().last().expect("a page").contains("4 accounts"),
        "{page}"
    );
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for subject in Subject::ALL {
        for width in [80u16, 120, 200] {
            for line in drawn_at(&fixture::view(), *subject, width).lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{} at {width} columns: {line}",
                    subject.name()
                );
            }
        }
    }
}
