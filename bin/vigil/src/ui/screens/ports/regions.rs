use ratatui::layout::Rect;

use super::showing::Showing;
use crate::ui::Look;

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

pub(super) fn split_top(area: Rect, showing: &Showing<'_>) -> (Option<Rect>, Option<Rect>, Rect) {
    if area.height < 3 {
        return (None, None, area);
    }
    let chooser = Rect { height: 1, ..area };
    let wanted = showing.search.typing() || showing.search.holding_back();
    if !wanted || area.height < 4 {
        return (
            Some(chooser),
            None,
            Rect {
                y: area.y + 1,
                height: area.height - 1,
                ..area
            },
        );
    }
    (
        Some(chooser),
        Some(Rect {
            y: area.y + 1,
            height: 1,
            ..area
        }),
        Rect {
            y: area.y + 2,
            height: area.height - 2,
            ..area
        },
    )
}

pub(super) fn split_bottom(area: Rect, look: Look, rows: usize) -> (Rect, Rect) {
    if area.height < 2 {
        return (area, Rect { height: 0, ..area });
    }
    let table = match look.interactive() {
        true => area.height - 1,
        false => (rows.max(2) as u16).saturating_add(1).min(area.height - 1),
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
