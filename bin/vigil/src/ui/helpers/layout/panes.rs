use ratatui::layout::Rect;

use crate::ui::{Look, Search};

const ROOM_FOR_THE_DEFINITION: u16 = 7;

pub fn menu(area: Rect) -> (Option<Rect>, Rect) {
    if area.height < 3 {
        return (None, area);
    }
    (Some(Rect { height: 1, ..area }), below(area, 1))
}

pub fn about(area: Rect, look: Look, lines: u16) -> (Option<Rect>, Rect) {
    let short = look.interactive() && area.height < ROOM_FOR_THE_DEFINITION + lines;
    if lines == 0 || short {
        return (None, area);
    }
    (
        Some(Rect {
            height: lines,
            ..area
        }),
        below(area, lines),
    )
}

pub fn search_box(area: Rect, search: &Search) -> (Option<Rect>, Rect) {
    if !(search.typing() || search.holding_back()) || area.height < 3 {
        return (None, area);
    }
    (Some(Rect { height: 1, ..area }), below(area, 1))
}

pub fn footer(area: Rect, look: Look, wanted: usize) -> (Rect, Rect) {
    if area.height < 2 {
        return (area, Rect { height: 0, ..area });
    }
    let table = match look.interactive() {
        true => area.height - 1,
        false => (wanted.max(2) as u16).min(area.height - 1),
    };
    (
        Rect {
            height: table,
            ..area
        },
        Rect {
            y: area.y + table,
            height: 1,
            ..area
        },
    )
}

fn below(area: Rect, lines: u16) -> Rect {
    Rect {
        y: area.y + lines,
        height: area.height.saturating_sub(lines),
        ..area
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture;

    #[test]
    fn a_pane_taken_off_the_top_leaves_the_rest_of_the_area_and_loses_no_row() {
        let area = Rect::new(0, 0, 80, 20);

        let (row, rest) = menu(area);

        let row = row.expect("room for a row of names");
        assert_eq!(row.height, 1);
        assert_eq!(rest.y, area.y + 1);
        assert_eq!(row.height + rest.height, area.height);
    }

    #[test]
    fn a_terminal_with_no_room_keeps_the_table_rather_than_the_decoration() {
        let cramped = Rect::new(0, 0, 80, 2);

        assert!(menu(cramped).0.is_none());
        assert!(
            about(Rect::new(0, 0, 80, 6), fixture::look(), 2)
                .0
                .is_none()
        );
    }

    #[test]
    fn a_file_gets_a_table_as_long_as_it_has_rows_and_a_terminal_gets_the_whole_height() {
        let area = Rect::new(0, 0, 80, 40);

        let (table, footer) = footer(area, fixture::look(), 3);

        assert_eq!(table.height, area.height - 1, "a terminal fills its window");
        assert_eq!(footer.height, 1);
    }
}
