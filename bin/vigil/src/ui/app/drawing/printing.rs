use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::app::App;

use crate::ui::details::section;
use crate::ui::screens::{home, pane};
use crate::ui::{Arrows, Search};

impl App {
    pub(in crate::ui::app) fn print_every_section(&self, area: Rect, buffer: &mut Buffer) {
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

    pub(in crate::ui::app) fn print_every_list(&self, area: Rect, buffer: &mut Buffer) {
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
            screen if screen.draws_a_reading() => self
                .every_shown_pane()
                .into_iter()
                .map(Band::Pane)
                .collect(),
            _ => Vec::new(),
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
        }
    }
}

#[derive(Clone, Copy)]
enum Band {
    Pane(usize),
}
