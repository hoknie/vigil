use ratatui::layout::Rect;

use crate::ui::{Filter, Look};

pub(super) fn split_top(area: Rect, filter: &Filter) -> (Option<Rect>, Rect) {
    let wanted = filter.search().typing() || filter.holding_back();
    if !wanted || area.height < 3 {
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

pub(super) fn split_bottom(
    area: Rect,
    look: Look,
    rows: usize,
    picking: bool,
) -> (Rect, Rect, Option<Rect>) {
    if area.height < 2 {
        return (area, Rect { height: 0, ..area }, None);
    }
    let bar = match picking && area.height >= 4 {
        true => Some(Rect {
            y: area.y + area.height - 1,
            height: 1,
            ..area
        }),
        false => None,
    };
    let room = area.height - u16::from(bar.is_some());
    let table = match look.interactive() {
        true => room - 1,
        false => (rows.max(2) as u16).saturating_add(1).min(room - 1),
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
        bar,
    )
}
