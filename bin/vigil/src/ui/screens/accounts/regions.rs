use ratatui::layout::Rect;

use super::row::Row;
use crate::ui::{Look, Notice, Search};

const ROOM_FOR_THE_DEFINITION: u16 = 7;

pub(super) fn split_menu(area: Rect) -> (Option<Rect>, Rect) {
    if area.height < 3 {
        return (None, area);
    }
    (
        Some(Rect { height: 1, ..area }),
        Rect {
            y: area.y + 1,
            height: area.height - 1,
            ..area
        },
    )
}

pub(super) fn split_about(area: Rect, look: Look, lines: u16) -> (Option<Rect>, Rect) {
    let short = look.interactive() && area.height < ROOM_FOR_THE_DEFINITION + lines;
    if lines == 0 || short {
        return (None, area);
    }
    (
        Some(Rect {
            height: lines,
            ..area
        }),
        Rect {
            y: area.y + lines,
            height: area.height - lines,
            ..area
        },
    )
}

pub(super) fn split_top(area: Rect, search: &Search) -> (Option<Rect>, Rect) {
    if !(search.typing() || search.holding_back()) || area.height < 3 {
        return (None, area);
    }
    (
        Some(Rect { height: 1, ..area }),
        Rect {
            y: area.y + 1,
            height: area.height - 1,
            ..area
        },
    )
}

pub(super) fn middle(notice: &Option<Notice>, look: Look, rows: &[Row<'_>], width: u16) -> usize {
    match notice {
        Some(notice) => notice.lines(look, width as usize).len(),
        None => rows.len() + 1,
    }
}

pub(super) fn split_bottom(area: Rect, look: Look, wanted: usize) -> (Rect, Rect) {
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
