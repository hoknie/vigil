use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::Arrows;
use crate::ui::app::App;
use crate::ui::details::section;
use crate::ui::helpers::layout::split;
use crate::ui::screens::home;
use crate::ui::theme::caption;

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
            home::render(&self.view, self.look, &showing, body, buffer);
            return;
        }

        let Some(split) = split::beside(body) else {
            let table = home::printed_height(&self.view, body.width)
                .min(body.height.saturating_sub(self.panel_wants(body)))
                .max(ROOM_FOR_THE_CURSOR)
                .min(body.height);
            let (list, rest) = (
                Rect {
                    height: table,
                    ..body
                },
                Rect {
                    y: body.y + table,
                    height: body.height.saturating_sub(table),
                    ..body
                },
            );
            home::render(&self.view, self.look, &showing, list, buffer);
            self.draw_section_panel(rest, buffer);
            return;
        };

        Paragraph::new(caption::render(
            self.look,
            "SECTIONS",
            "",
            caption::Keys::Here,
            split.list.width as usize,
        ))
        .render(
            Rect {
                height: 1,
                ..split.list
            },
            buffer,
        );
        home::render(
            &self.view,
            self.look,
            &showing,
            Rect {
                y: split.list.y + 1,
                height: split.list.height.saturating_sub(1),
                ..split.list
            },
            buffer,
        );
        Paragraph::new(vec![Line::raw(" │ "); split.rule.height as usize])
            .style(self.look.palette.border())
            .render(split.rule, buffer);
        self.draw_section_panel(split.panel, buffer);
    }

    fn panel_wants(&self, body: Rect) -> u16 {
        let rows = home::rows(&self.view);
        let row = rows.get(self.nav.sections.at());
        section::height(row, self.look, self.look.text_width(body.width)) as u16 + 1
    }

    fn draw_section_panel(&self, area: Rect, buffer: &mut Buffer) {
        if area.height < 2 {
            return;
        }
        let rows = home::rows(&self.view);
        let row = rows.get(self.nav.sections.at());
        Paragraph::new(caption::render(
            self.look,
            "THE SELECTED SECTION",
            &caption::scrolled(
                0,
                area.height as usize - 1,
                section::height(row, self.look, self.look.text_width(area.width)),
            ),
            caption::Keys::Sole,
            area.width as usize,
        ))
        .render(Rect { height: 1, ..area }, buffer);
        section::render(
            row,
            self.look,
            0,
            Rect {
                y: area.y + 1,
                height: area.height - 1,
                ..area
            },
            buffer,
        );
    }
}
