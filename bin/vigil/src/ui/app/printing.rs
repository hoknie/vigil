use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::App;

use crate::ui::details::section;
use crate::ui::screens::startup as starting;
use crate::ui::screens::{home, pane, programs};
use crate::ui::{Arrows, Program, Screen, Search, Startup, holding};

impl App {
    pub(super) fn print_every_section(&self, area: Rect, buffer: &mut Buffer) {
        let table = home::printed_height(&self.view, area.width).min(area.height);
        home::render(
            &self.view,
            self.look,
            &home::Showing {
                cursor: 0,
                arrows: Arrows::Away,
            },
            Rect {
                height: table,
                ..area
            },
            buffer,
        );

        let mut top = area.y + table;
        let bottom = area.y + area.height;
        let width = self.look.text_width(area.width);

        for row in home::rows(&self.view) {
            if top >= bottom || row.standing.note.is_none() {
                continue;
            }
            let wanted = section::height(Some(&row), self.look, width) as u16 + 1;
            let room = Rect {
                y: top,
                height: wanted.min(bottom - top),
                ..area
            };
            section::render(Some(&row), self.look, 0, room, buffer);
            top += room.height;
        }
    }

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
            screen if holding(screen.name()).is_some() => {
                self.shown_panes().into_iter().map(Band::Pane).collect()
            }
            Screen::Programs => Program::ALL.iter().copied().map(Band::Program).collect(),
            _ => Startup::on(&self.view)
                .into_iter()
                .map(Band::Startup)
                .collect(),
        }
    }

    fn printed_height(&self, band: Band, search: &Search, width: u16) -> u16 {
        match band {
            Band::Pane(at) => match (self.section(), self.printing_pane(at, search)) {
                (Some(section), Some(showing)) => {
                    pane::printed_height(&self.view, self.look, section.as_ref(), &showing, width)
                }
                _ => 0,
            },
            Band::Program(program) => programs::printed_height(&self.view, program, search, width),
            Band::Startup(list) => {
                starting::printed_height(&self.view, &starting::Showing::plain(list, search), width)
            }
        }
    }

    fn print_one_list(&self, band: Band, search: &Search, area: Rect, buffer: &mut Buffer) {
        match band {
            Band::Pane(at) => {
                if let (Some(section), Some(showing)) =
                    (self.section(), self.printing_pane(at, search))
                {
                    pane::render(
                        &self.view,
                        self.look,
                        section.as_ref(),
                        &showing,
                        area,
                        buffer,
                    );
                }
            }
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
    Pane(usize),
    Program(Program),
    Startup(Startup),
}
