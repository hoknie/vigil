use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use super::App;

impl App {
    pub fn page(&self) -> Page<'_> {
        Page { app: self }
    }
}

pub struct Page<'a> {
    app: &'a App,
}

impl Widget for Page<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.app.draw(area, buffer);
    }
}
