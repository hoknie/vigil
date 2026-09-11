use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::App;

use crate::ui::chrome::frame;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::help;
use crate::ui::details::{account, program, socket, startup};
use crate::ui::helpers::finding::diff;
use crate::ui::helpers::layout::split;
use crate::ui::helpers::words::unreachable;
use crate::ui::screens::startup as starting;
use crate::ui::screens::{accounts, findings, home, ports, programs, summary};
use crate::ui::theme::caption;
use crate::ui::{Arrows, Level, Screen};

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
            },
            area,
            buffer,
        );
        self.body.set(body);

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
            Screen::Home => home::render(
                &self.view,
                self.look,
                &home::Showing {
                    cursor: self.nav.sections.at(),
                    arrows: Arrows::at(self.level),
                },
                body,
                buffer,
            ),
            Screen::Summary => {
                summary::render(&self.view, self.look, self.nav.summary.top(), body, buffer)
            }
            _ => self.draw_listed(body, buffer),
        }
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
            Screen::Ports => ports::render(&self.view, self.look, &self.listening(), area, buffer),
            Screen::Accounts => match self.look.interactive() {
                true => accounts::render(&self.view, self.look, &self.showing(), area, buffer),
                false => self.print_every_list(area, buffer),
            },
            Screen::Programs => match self.look.interactive() {
                true => programs::render(&self.view, self.look, &self.running(), area, buffer),
                false => self.print_every_list(area, buffer),
            },
            Screen::Startup => match self.look.interactive() {
                true => starting::render(&self.view, self.look, &self.starting(), area, buffer),
                false => self.print_every_list(area, buffer),
            },
            _ => findings::render(
                &self.view,
                &self.filter,
                self.look,
                self.nav.findings.at(),
                self.level == Level::List,
                area,
                buffer,
            ),
        }
    }

    pub(super) fn draw_detail(&self, area: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            Screen::Ports => {
                let rows = self.ports_rows();
                socket::render(
                    rows.get(self.nav.lists.ports.at()),
                    self.look,
                    self.nav.difference.top(),
                    area,
                    buffer,
                )
            }
            Screen::Accounts => {
                let rows = self.accounts_rows();
                account::render(
                    rows.get(self.nav.lists.accounts.at()),
                    &self.view,
                    self.look,
                    self.nav.difference.top(),
                    area,
                    buffer,
                )
            }
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
            Screen::Ports => self.nav.lists.ports.showing().caption(),
            Screen::Accounts => self.nav.lists.accounts.showing().caption(),
            Screen::Programs => self.nav.lists.programs.showing().caption(),
            Screen::Startup => self.nav.lists.startup.showing().caption(),
            _ => "FINDINGS",
        }
    }

    pub(super) fn detail_caption(&self) -> &'static str {
        match self.nav.at() {
            Screen::Ports => "THE SELECTED SOCKET",
            Screen::Accounts => self.nav.lists.accounts.showing().detail(),
            Screen::Programs => self.nav.lists.programs.showing().detail(),
            Screen::Startup => self.nav.lists.startup.showing().detail(),
            _ => "THE SELECTED FINDING",
        }
    }
}
