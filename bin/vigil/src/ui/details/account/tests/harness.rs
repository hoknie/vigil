use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::details::account::render;
use crate::ui::helpers::words::text as page;
use crate::ui::screens::accounts;
use crate::ui::{Search, Subject, View, fixture};

pub(super) fn drawn(view: &View, key: &str, width: u16) -> String {
    let rows = accounts::rows(view, Subject::holding(key), &Search::default());
    let row = rows
        .iter()
        .find(|row| row.key == key)
        .unwrap_or_else(|| panic!("{key} is not a row"));
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 60));
    render(
        Some(row),
        view,
        fixture::look(),
        0,
        buffer.area,
        &mut buffer,
    );
    page::to_text(&buffer)
}
