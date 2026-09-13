use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::App;

use crate::ui::chrome::chooser;
use crate::ui::chrome::frame;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::help;
use crate::ui::details::{pieces, program, reading, section, startup};
use crate::ui::helpers::finding::diff;
use crate::ui::helpers::layout::split;
use crate::ui::helpers::words::unreachable;
use crate::ui::screens::startup as starting;
use crate::ui::screens::{findings, home, pane, programs, summary, system};
use crate::ui::theme::caption;
use crate::ui::{Arrows, Level, Screen, holding};

const ROOM_FOR_THE_CURSOR: u16 = 8;

impl App {
    pub fn draw(&self, area: Rect, buffer: &mut Buffer) {
        let body = frame::render(
            self.look,
            self.nav.at(),
            &self.view,
            frame::Hints {
                typing: self.typing(),
                level: self.level,
                message: self.message.as_deref(),
                back: self.back(),
                panel: self.detail_showing(self.body.get()) || self.showing_why(),
                choosing: self.choosing(),
            },
            area,
            buffer,
        );
        self.body.set(body);

        let body = match chooser::height(&self.chooser, self.look, body.width) {
            0 => body,
            tall => {
                let tall = tall.min(body.height);
                chooser::render(
                    &self.chooser,
                    self.look,
                    Rect {
                        height: tall,
                        ..body
                    },
                    buffer,
                );
                Rect {
                    y: body.y + tall,
                    height: body.height.saturating_sub(tall),
                    ..body
                }
            }
        };

        if !self.view.has_reading() {
            unreachable::render(&self.view, self.look, body, buffer);
        } else {
            self.draw_screen(body, buffer);
        }

        if self.helping {
            help::render(self.look, area, buffer);
        }
    }

    pub(super) fn back(&self) -> Back {
        match (self.nav.at(), self.nav.came_from()) {
            (Screen::Home, _) => Back::Nowhere,
            (_, Some(_)) => Back::Finding,
            (_, None) => Back::MainScreen,
        }
    }

    pub(super) fn draw_screen(&self, body: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            Screen::Home => self.draw_home(body, buffer),
            Screen::Summary => summary::render(
                &self.view,
                self.look,
                self.nav.summary.top(),
                self.saying(),
                self.gone.as_ref(),
                body,
                buffer,
            ),
            _ => self.draw_listed(body, buffer),
        }
    }

    pub(super) fn draw_home(&self, body: Rect, buffer: &mut Buffer) {
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

    pub(super) fn draw_listed(&self, body: Rect, buffer: &mut Buffer) {
        let laid_out = split::layout(body, self.detail_showing(body));

        if let Some(area) = laid_out.list {
            self.draw_list(area, buffer);
        }
        if let Some(area) = laid_out.list_caption {
            Paragraph::new(caption::render(
                self.look,
                self.list_caption(),
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
                match (laid_out.list.is_some(), self.level) {
                    (false, _) => caption::Keys::Sole,
                    (true, Level::Detail) => caption::Keys::Here,
                    (true, _) => caption::Keys::Elsewhere,
                },
                over.width as usize,
            ))
            .render(over, buffer);
        }
        self.draw_detail(area, buffer);
    }

    pub(super) fn draw_list(&self, area: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            screen if holding(screen.name()).is_some() => {
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
            Screen::Programs => match self.look.interactive() {
                true => programs::render(&self.view, self.look, &self.running(), area, buffer),
                false => self.print_every_list(area, buffer),
            },
            Screen::Startup => match self.look.interactive() {
                true => starting::render(&self.view, self.look, &self.starting(), area, buffer),
                false => self.print_every_list(area, buffer),
            },
            Screen::System => system::render(&self.view, self.look, &self.made_of(), area, buffer),
            _ => findings::render(&self.view, self.look, &self.found(), area, buffer),
        }
    }

    pub(super) fn draw_detail(&self, area: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            screen if holding(screen.name()).is_some() => pieces::render(
                &self.pane_detail(),
                self.look,
                self.nav.difference.top(),
                area,
                buffer,
            ),
            Screen::Programs => {
                let rows = self.programs_rows();
                program::render(
                    rows.get(self.nav.lists.programs.at()),
                    self.running_list(),
                    self.look,
                    self.nav.difference.top(),
                    area,
                    buffer,
                )
            }
            Screen::Startup => {
                let rows = self.startup_rows();
                startup::render(
                    rows.get(self.nav.lists.startup.at()),
                    self.look,
                    self.nav.difference.top(),
                    area,
                    buffer,
                )
            }
            Screen::System => reading::render(
                self.reading_subject(),
                self.look,
                self.nav.difference.top(),
                area,
                buffer,
            ),
            _ => diff::render(
                self.selected_finding(),
                self.look,
                self.nav.difference.top(),
                area,
                buffer,
            ),
        }
    }

    pub(super) fn list_caption(&self) -> &'static str {
        match self.nav.at() {
            Screen::Home => "SECTIONS",
            screen if holding(screen.name()).is_some() => self.pane_caption(),
            Screen::Programs => self.nav.lists.programs.showing().caption(),
            Screen::Startup => self.nav.lists.startup.showing().caption(),
            Screen::System => self.nav.lists.system.showing().caption(),
            _ => "FINDINGS",
        }
    }

    pub(super) fn detail_caption(&self) -> &'static str {
        match self.nav.at() {
            screen if holding(screen.name()).is_some() => self.pane_detail_caption(),
            Screen::Programs => self.nav.lists.programs.showing().detail(),
            Screen::Startup => self.nav.lists.startup.showing().detail(),
            Screen::System => self.nav.lists.system.showing().detail(),
            _ => "THE SELECTED FINDING",
        }
    }
}
