use ratatui::layout::Rect;

use super::App;

use crate::ui::details::{account, program, socket, startup};
use crate::ui::helpers::finding::diff;
use crate::ui::helpers::layout::split;
use crate::ui::{Level, Screen};

impl App {
    pub(super) fn has_detail(&self) -> bool {
        match self.nav.at() {
            Screen::Findings => self.selected_finding().is_some(),
            Screen::Ports => !self.ports_keys().is_empty(),
            Screen::Accounts => !self.accounts_keys().is_empty(),
            Screen::Programs => !self.programs_keys().is_empty(),
            Screen::Startup => !self.startup_keys().is_empty(),
            Screen::Home | Screen::Summary => false,
        }
    }

    pub(super) fn detail_showing(&self, body: Rect) -> bool {
        self.detail_open && (split::beside(body).is_some() || self.level == Level::Detail)
    }

    pub(super) fn detail_area(&self) -> Option<Rect> {
        split::layout(self.body.get(), self.detail_showing(self.body.get())).detail()
    }

    pub(super) fn detail_height(&self, area: Rect) -> usize {
        let width = self.look.text_width(area.width);
        match self.nav.at() {
            Screen::Ports => {
                let rows = self.ports_rows();
                socket::height(rows.get(self.nav.lists.ports.at()), self.look, width)
            }
            Screen::Accounts => {
                let rows = self.accounts_rows();
                account::height(
                    rows.get(self.nav.lists.accounts.at()),
                    &self.view,
                    self.look,
                    width,
                )
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
            _ => diff::height(self.selected_finding(), self.look, width),
        }
    }
}
