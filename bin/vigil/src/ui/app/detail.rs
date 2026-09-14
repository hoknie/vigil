use ratatui::layout::Rect;

use super::App;

use crate::ui::details::pieces;
use crate::ui::helpers::finding::diff;
use crate::ui::helpers::layout::split;
use crate::ui::{Level, Screen, holding};

impl App {
    pub(super) fn has_detail(&self) -> bool {
        match self.nav.at() {
            Screen::FINDINGS => self.selected_finding().is_some(),
            screen if holding(screen.name()).is_some() => !self.pane_keys().is_empty(),
            _ => false,
        }
    }

    pub(super) fn detail_showing(&self, body: Rect) -> bool {
        if self.nav.at() == Screen::HOME {
            return self.detail_open;
        }
        self.detail_open && (split::beside(body).is_some() || self.level == Level::Detail)
    }

    pub(super) fn detail_area(&self) -> Option<Rect> {
        split::layout(self.body.get(), self.detail_showing(self.body.get())).detail()
    }

    pub(super) fn detail_height(&self, area: Rect) -> usize {
        let width = self.look.text_width(area.width);
        match self.nav.at() {
            screen if holding(screen.name()).is_some() => {
                pieces::height(&self.pane_detail(), self.acts(), self.look, width)
            }
            _ => diff::height(self.selected_finding(), self.look, width),
        }
    }
}
