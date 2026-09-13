use ratatui::layout::Rect;

use super::App;

use crate::ui::details::{pieces, program, reading, startup};
use crate::ui::helpers::finding::diff;
use crate::ui::helpers::layout::split;
use crate::ui::{Level, Screen, holding};

impl App {
    pub(super) fn has_detail(&self) -> bool {
        match self.nav.at() {
            Screen::Findings => self.selected_finding().is_some(),
            screen if holding(screen.name()).is_some() => !self.pane_keys().is_empty(),
            Screen::Programs => !self.programs_keys().is_empty(),
            Screen::Startup => !self.startup_keys().is_empty(),
            Screen::System => !self.system_keys().is_empty(),
            _ => false,
        }
    }

    pub(super) fn detail_showing(&self, body: Rect) -> bool {
        if self.nav.at() == Screen::Home {
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
                pieces::height(&self.pane_detail(), self.look, width)
            }
            Screen::Programs => {
                let rows = self.programs_rows();
                program::height(
                    rows.get(self.nav.lists.programs.at()),
                    self.nav.lists.programs.showing(),
                    self.look,
                    width,
                )
            }
            Screen::Startup => {
                let rows = self.startup_rows();
                startup::height(rows.get(self.nav.lists.startup.at()), self.look, width)
            }
            Screen::System | Screen::Containers => {
                reading::height(self.reading_subject(), self.look, width)
            }
            _ => diff::height(self.selected_finding(), self.look, width),
        }
    }
}
