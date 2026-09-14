use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::app::App;
use crate::ui::details::pieces;
use crate::ui::helpers::finding::diff;
use crate::ui::helpers::layout::split;
use crate::ui::screens::{findings, pane};
use crate::ui::theme::caption;
use crate::ui::{Level, Screen};

impl App {
    pub(in crate::ui::app) fn draw_listed(&self, body: Rect, buffer: &mut Buffer) {
        let laid_out = split::layout(body, self.detail_showing(body));

        if let Some(area) = laid_out.list {
            self.draw_list(area, buffer);
        }
        if let Some(area) = laid_out.list_caption {
            Paragraph::new(caption::render(
                self.look,
                &self.list_caption(),
                "",
                match self.level {
                    Level::List => caption::Keys::Here,
                    _ => caption::Keys::Elsewhere,
                },
                area.width as usize,
            ))
            .render(area, buffer);
        }
        if let Some(area) = laid_out.rule {
            Paragraph::new(vec![Line::raw(" │ "); area.height as usize])
                .style(self.look.palette.border())
                .render(area, buffer);
        }

        let Some(area) = laid_out.detail() else {
            return;
        };
        if let Some(over) = laid_out.detail_caption {
            let total = self.detail_height(area);
            Paragraph::new(caption::render(
                self.look,
                self.detail_caption(),
                &caption::scrolled(self.nav.difference.top(), area.height as usize, total),
                match (laid_out.list.is_some(), self.level, self.button_at()) {
                    (false, _, _) => caption::Keys::Sole,
                    (true, Level::Detail, None) => caption::Keys::Here,
                    (true, _, _) => caption::Keys::Elsewhere,
                },
                over.width as usize,
            ))
            .render(over, buffer);
        }
        self.draw_detail(area, buffer);
    }

    pub(in crate::ui::app) fn draw_list(&self, area: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            screen if screen.draws_a_reading() => {
                if !self.look.interactive() && self.shown_panes().len() > 1 {
                    self.print_every_list(area, buffer);
                    return;
                }
                if let (Some(section), Some(showing)) = (self.section(), self.showing_pane()) {
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
            _ => findings::render(&self.view, self.look, &self.found(), area, buffer),
        }
    }

    pub(in crate::ui::app) fn draw_detail(&self, area: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            screen if screen.draws_a_reading() => pieces::render(
                &self.pane_detail(),
                self.acts(),
                self.button_at(),
                self.look,
                self.nav.difference.top(),
                area,
                buffer,
            ),
            _ => diff::render(
                self.selected_finding(),
                self.look,
                self.nav.difference.top(),
                self.picked.count(),
                area,
                buffer,
            ),
        }
    }

    pub(in crate::ui::app) fn list_caption(&self) -> String {
        match self.nav.at() {
            Screen::HOME => "SECTIONS".to_string(),
            screen if screen.draws_a_reading() => self.pane_caption(),
            _ => "FINDINGS".to_string(),
        }
    }

    pub(in crate::ui::app) fn detail_caption(&self) -> &'static str {
        match self.nav.at() {
            screen if screen.draws_a_reading() => self.pane_detail_caption(),
            _ => "THE SELECTED FINDING",
        }
    }
}
