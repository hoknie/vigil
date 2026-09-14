use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{
    HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table,
    TableState,
};

use crate::ui::Look;
use crate::ui::helpers::layout::scroll;

const GUTTER: u16 = 1;

const MARKER: &str = " > ";

const COLD_MARKER: &str = " · ";

pub fn column_widths(look: Look, widths: &[Constraint], area: Rect) -> Vec<usize> {
    let gutter = match look.interactive() {
        true => GUTTER.min(area.width),
        false => 0,
    };
    let [_marker, columns] = Layout::horizontal([
        Constraint::Length(MARKER.chars().count() as u16),
        Constraint::Fill(0),
    ])
    .areas(Rect::new(0, 0, area.width - gutter, 1));

    Layout::horizontal(widths.to_vec())
        .flex(Flex::Start)
        .spacing(1)
        .split(columns)
        .iter()
        .map(|column| column.width as usize)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Where {
    pub at: usize,
    pub focused: bool,
}

#[derive(Clone, Copy)]
pub struct Rows<'a> {
    pub total: usize,
    pub drawn: &'a dyn Fn(usize) -> Row<'static>,
}

pub fn render(
    look: Look,
    header: Row<'static>,
    rows: Rows<'_>,
    widths: &[Constraint],
    cursor: Where,
    area: Rect,
    buffer: &mut Buffer,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let page = area.height.saturating_sub(1) as usize;
    let total = rows.total;
    let gutter = match look.interactive() {
        true => GUTTER.min(area.width),
        false => 0,
    };
    let first = scroll::window(cursor.at, total, page);
    let on_the_screen: Vec<Row<'static>> =
        (first..(first + page).min(total)).map(rows.drawn).collect();

    let table = Table::new(on_the_screen, widths.to_vec())
        .header(header.style(look.palette.quiet()))
        .column_spacing(1)
        .row_highlight_style(match cursor.focused {
            true => look.palette.selected(),
            false => look.palette.marked(),
        })
        .highlight_symbol(match cursor.focused {
            true => MARKER,
            false => COLD_MARKER,
        })
        .highlight_spacing(HighlightSpacing::Always);

    let mut state = TableState::new()
        .with_offset(0)
        .with_selected(match look.interactive() {
            true => Some(cursor.at.saturating_sub(first)),
            false => None,
        });

    StatefulWidget::render(
        table,
        Rect {
            width: area.width - gutter,
            ..area
        },
        buffer,
        &mut state,
    );

    if gutter > 0 && total > page {
        let mut bar = ScrollbarState::new(total.saturating_sub(page)).position(first);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(look.palette.border())
            .begin_symbol(None)
            .end_symbol(None)
            .render(
                Rect {
                    y: area.y + 1,
                    height: area.height - 1,
                    ..area
                },
                buffer,
                &mut bar,
            );
    }
}

#[cfg(test)]
mod tests {
    use ratatui::widgets::Row;

    use super::*;
    use crate::ui::helpers::words::text;
    use crate::ui::{Audience, Look, fixture};

    fn row(number: usize) -> Row<'static> {
        Row::new(vec![format!("row {number}")])
    }

    fn rows(count: usize) -> Rows<'static> {
        Rows {
            total: count,
            drawn: &row,
        }
    }

    #[test]
    fn only_the_rows_on_the_screen_are_built_however_long_the_list_is() {
        let built = std::cell::Cell::new(0usize);
        let counted = |number: usize| {
            built.set(built.get() + 1);
            row(number)
        };
        let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 8));

        render(
            fixture::look(),
            Row::new(vec!["NAME"]),
            Rows {
                total: 10_000,
                drawn: &counted,
            },
            &[Constraint::Fill(1)],
            Where {
                at: 9_000,
                focused: true,
            },
            buffer.area,
            &mut buffer,
        );

        assert!(
            built.get() <= 7,
            "{} rows were built for a screen that shows seven: the cells of a row can cost a \
             walk over the whole reading, and building every row of a long list on every \
             keypress is the lag a person feels on a list of four hundred",
            built.get()
        );
        assert!(text::to_text(&buffer).contains("row 9000"));
    }

    fn drawn(look: Look, count: usize, cursor: usize, height: u16) -> String {
        drawn_with_focus(look, count, cursor, height, true)
    }

    fn drawn_with_focus(
        look: Look,
        count: usize,
        cursor: usize,
        height: u16,
        focused: bool,
    ) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 24, height));
        render(
            look,
            Row::new(vec!["NAME"]),
            rows(count),
            &[Constraint::Fill(1)],
            Where {
                at: cursor,
                focused,
            },
            buffer.area,
            &mut buffer,
        );
        text::to_text(&buffer)
    }

    #[test]
    fn the_selected_row_is_marked_with_a_character_and_not_only_a_highlight() {
        let page = drawn(fixture::look(), 5, 2, 8);

        assert!(
            page.lines().any(|line| line.starts_with(" > row 2")),
            "a highlight alone is invisible on a monochrome terminal: {page}"
        );
    }

    #[test]
    fn a_table_the_arrows_are_not_moving_keeps_its_row_marked_and_loses_the_arrow() {
        let cold = drawn_with_focus(fixture::look(), 5, 2, 8, false);

        assert!(
            cold.lines().any(|line| line.starts_with(" · row 2")),
            "the row is no longer marked at all: {cold}"
        );
        assert!(
            !cold.contains(" > "),
            "and must not still offer the cursor: {cold}"
        );
    }

    #[test]
    fn the_window_follows_the_cursor_down_a_list_that_does_not_fit() {
        let page = drawn(fixture::look(), 100, 99, 6);

        assert!(page.contains("row 99"), "{page}");
        assert!(!page.contains("row 0\n"), "{page}");
    }

    #[test]
    fn a_list_longer_than_the_screen_says_so_with_a_bar_down_the_side() {
        let short = drawn(fixture::look(), 3, 0, 8);
        let long = drawn(fixture::look(), 100, 0, 8);

        assert!(
            !short.contains('█') && !short.contains('║'),
            "nothing to scroll, nothing to draw: {short}"
        );
        assert!(
            long.contains('█') || long.contains('║'),
            "a list with more in it has to say so: {long}"
        );
    }

    #[test]
    fn a_file_gets_the_table_without_the_cursor_or_the_bar() {
        let page = drawn(
            Look::new(fixture::monochrome(), Audience::Script),
            100,
            0,
            8,
        );

        assert!(!page.contains(" > "), "a file cannot move a cursor: {page}");
        assert!(!page.contains('█'), "or drag a bar: {page}");
        assert!(page.contains("row 0"), "{page}");
    }

    #[test]
    fn a_window_with_no_room_in_it_draws_nothing_rather_than_dividing_by_it() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 8));
        render(
            fixture::look(),
            Row::new(vec!["NAME"]),
            rows(4),
            &[Constraint::Fill(1)],
            Where {
                at: 0,
                focused: true,
            },
            Rect::new(0, 0, 0, 0),
            &mut buffer,
        );

        assert_eq!(text::to_text(&buffer), "");
    }
}
