use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::App;

use crate::ui::screens::startup as starting;
use crate::ui::screens::{accounts, programs};
use crate::ui::{Arrows, Program, Screen, Search, Startup, Subject};

impl App {
    pub(super) fn print_every_list(&self, area: Rect, buffer: &mut Buffer) {
        let mut top = area.y;
        let bottom = area.y + area.height;
        let search = Search::default();

        for band in self.every_list() {
            if top >= bottom {
                return;
            }
            let height = self
                .printed_height(band, &search, area.width)
                .min(bottom - top);
            let room = Rect {
                y: top,
                height,
                ..area
            };
            self.print_one_list(band, &search, room, buffer);
            top += height;
        }
    }

    fn every_list(&self) -> Vec<Band> {
        match self.nav.at() {
            Screen::Accounts => Subject::on(&self.view)
                .into_iter()
                .map(Band::Account)
                .collect(),
            Screen::Programs => Program::ALL.iter().copied().map(Band::Program).collect(),
            _ => Startup::on(&self.view)
                .into_iter()
                .map(Band::Startup)
                .collect(),
        }
    }

    fn printed_height(&self, band: Band, search: &Search, width: u16) -> u16 {
        match band {
            Band::Account(subject) => {
                accounts::printed_height(&self.view, self.look, subject, search, width)
            }
            Band::Program(program) => programs::printed_height(&self.view, program, search, width),
            Band::Startup(list) => {
                starting::printed_height(&self.view, &starting::Showing::plain(list, search), width)
            }
        }
    }

    fn print_one_list(&self, band: Band, search: &Search, area: Rect, buffer: &mut Buffer) {
        match band {
            Band::Account(subject) => accounts::render(
                &self.view,
                self.look,
                &accounts::Showing {
                    subject,
                    search,
                    cursor: 0,
                    elsewhere: 0,
                    arrows: Arrows::Away,
                },
                area,
                buffer,
            ),
            Band::Program(program) => programs::render(
                &self.view,
                self.look,
                &programs::Showing {
                    program,
                    search,
                    cursor: 0,
                    elsewhere: 0,
                    arrows: Arrows::Away,
                },
                area,
                buffer,
            ),
            Band::Startup(list) => starting::render(
                &self.view,
                self.look,
                &starting::Showing::plain(list, search),
                area,
                buffer,
            ),
        }
    }
}

#[derive(Clone, Copy)]
enum Band {
    Account(Subject),
    Program(Program),
    Startup(Startup),
}
