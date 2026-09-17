use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::ui::app::App;
use crate::ui::details::section;
use crate::ui::helpers::layout::{footing, split};
use crate::ui::screens::home;
use crate::ui::theme::{caption, panel};
use crate::ui::{Arrows, Target};

const ROOM_FOR_THE_CURSOR: u16 = 8;

impl App {
    pub(in crate::ui::app) fn draw_home(&self, body: Rect, buffer: &mut Buffer) {
        let showing = home::Showing {
            cursor: self.nav.sections.at(),
            arrows: Arrows::at(self.level),
        };

        if !self.look.interactive() {
            self.print_every_section(body, buffer);
            return;
        }

        if !self.detail_showing(body) {
            self.pointer.put(body, Target::List);
            let rows = home::render(
                &self.view,
                self.look,
                &showing,
                footing::onto_the_edge(self.look, body, buffer.area),
                buffer,
            );
            self.rows_drawn(rows);
            return;
        }

        let laid_out = split::layout(body, true, true);
        let (list, panel) = match (laid_out.list_frame, laid_out.detail_frame) {
            (Some(list), Some(panel)) => (list, panel),
            _ => {
                let framed = (home::printed_height(&self.view, body.width) + 1)
                    .min(body.height.saturating_sub(self.panel_wants(body)))
                    .max(ROOM_FOR_THE_CURSOR)
                    .min(body.height);
                (
                    Rect {
                        height: framed,
                        ..body
                    },
                    Rect {
                        y: body.y + framed,
                        height: body.height.saturating_sub(framed),
                        ..body
                    },
                )
            }
        };

        panel::block(self.look, "SECTIONS", caption::Keys::Here, true).render(list, buffer);
        self.pointer.put(list, Target::List);
        let rows = home::render(
            &self.view,
            self.look,
            &showing,
            footing::onto_the_edge(self.look, split::inside(list), buffer.area),
            buffer,
        );
        self.rows_drawn(rows);
        self.draw_section_panel(panel, buffer);
    }

    fn panel_wants(&self, body: Rect) -> u16 {
        let rows = home::rows(&self.view);
        let row = rows.get(self.nav.sections.at());
        section::height(row, self.look, self.look.text_width(body.width)) as u16 + 2
    }

    fn draw_section_panel(&self, area: Rect, buffer: &mut Buffer) {
        if area.height < 3 {
            return;
        }
        let rows = home::rows(&self.view);
        let row = rows.get(self.nav.sections.at());
        let inside = split::inside(area);
        self.pointer.put(area, Target::Detail);
        panel::tail(
            panel::block(
                self.look,
                "THE SELECTED SECTION",
                caption::Keys::Sole,
                false,
            ),
            self.look,
            &caption::scrolled(
                0,
                inside.height as usize,
                section::height(row, self.look, self.look.text_width(inside.width)),
            ),
        )
        .render(area, buffer);
        section::render(row, self.look, 0, inside, buffer);
    }
}
