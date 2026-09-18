use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::app::App;
use crate::ui::details::pieces;
use crate::ui::helpers::finding::diff;
use crate::ui::helpers::layout::{footing, split};
use crate::ui::screens::{findings, pane};
use crate::ui::theme::{caption, panel};
use crate::ui::{Level, Screen, Target};

impl App {
    pub(in crate::ui::app) fn draw_listed(&self, body: Rect, buffer: &mut Buffer) {
        let laid_out = split::layout(body, self.detail_showing(body), self.look.interactive());

        if let Some(area) = laid_out.list_caption {
            Paragraph::new(caption::render(
                self.look,
                &self.list_caption(),
                "",
                self.list_keys(),
                area.width as usize,
            ))
            .render(area, buffer);
        }
        if let Some(area) = laid_out.list_frame {
            panel::block(
                self.look,
                &self.list_caption(),
                self.list_keys(),
                self.level != Level::Detail,
            )
            .render(area, buffer);
        }
        if let Some(area) = laid_out.list {
            self.pointer
                .put(laid_out.list_frame.unwrap_or(area), Target::List);
            self.draw_list(footing::onto_the_edge(self.look, area, buffer.area), buffer);
        }
        if let Some(area) = laid_out.rule {
            Paragraph::new(vec![Line::raw(" │ "); area.height as usize])
                .style(self.look.palette.border())
                .render(area, buffer);
        }

        let Some(area) = laid_out.detail() else {
            return;
        };
        let keys = match (laid_out.list.is_some(), self.level, self.button_at()) {
            (false, _, None) if laid_out.detail_frame.is_some() => caption::Keys::Here,
            (false, _, _) => caption::Keys::Sole,
            (true, Level::Detail, None) => caption::Keys::Here,
            (true, _, _) => caption::Keys::Elsewhere,
        };
        let scrolled = caption::scrolled(
            self.nav.difference.top(),
            area.height as usize,
            self.detail_height(area),
        );
        if let Some(over) = laid_out.detail_caption {
            Paragraph::new(caption::render(
                self.look,
                self.detail_caption(),
                &scrolled,
                keys,
                over.width as usize,
            ))
            .render(over, buffer);
        }
        if let Some(frame) = laid_out.detail_frame {
            panel::tail(
                panel::block(
                    self.look,
                    self.detail_caption(),
                    keys,
                    self.level == Level::Detail,
                ),
                self.look,
                &scrolled,
            )
            .render(frame, buffer);
        }
        self.pointer
            .put(laid_out.detail_frame.unwrap_or(area), Target::Detail);
        self.draw_detail(area, buffer);
    }

    pub(in crate::ui::app) fn rows_drawn(&self, rows: Vec<(usize, Rect)>) {
        for (at, drawn) in rows {
            self.pointer.put(drawn, Target::Row(at));
        }
    }

    fn list_keys(&self) -> caption::Keys {
        match self.level {
            Level::List => caption::Keys::Here,
            _ => caption::Keys::Elsewhere,
        }
    }

    pub(in crate::ui::app) fn draw_list(&self, area: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            screen if screen.draws_a_reading() => {
                if !self.look.interactive() && self.every_shown_pane().len() > 1 {
                    self.print_every_list(area, buffer);
                    return;
                }
                if let (Some(section), Some(showing)) = (self.section(), self.showing_pane()) {
                    let showing = pane::Showing {
                        listed: Some(self.pane_rows()),
                        tally: Some(self.pane_tally()),
                        ..showing
                    };
                    let placed = pane::render(
                        &self.view,
                        self.look,
                        section.as_ref(),
                        &showing,
                        area,
                        buffer,
                    );
                    for (at, drawn) in placed.groups {
                        self.pointer.put(drawn, Target::Group(at));
                    }
                    for (at, drawn) in placed.names {
                        self.pointer.put(drawn, Target::Pane(at));
                    }
                    self.rows_drawn(placed.rows);
                }
            }
            _ => {
                let rows = findings::render(&self.view, self.look, &self.found(), area, buffer);
                self.rows_drawn(rows);
            }
        }
    }

    pub(in crate::ui::app) fn draw_detail(&self, area: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            screen if screen.draws_a_reading() => {
                for (at, drawn) in pieces::render(
                    &self.pane_detail(),
                    self.acts(),
                    self.button_at(),
                    self.look,
                    self.nav.difference.top(),
                    area,
                    buffer,
                ) {
                    self.pointer.put(drawn, Target::Button(at));
                }
            }
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
