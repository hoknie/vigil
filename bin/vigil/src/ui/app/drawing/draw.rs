use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::Screen;
use crate::ui::app::App;
use crate::ui::chrome::chooser;
use crate::ui::chrome::frame;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::help;
use crate::ui::chrome::paper;
use crate::ui::helpers::words::unreachable;
use crate::ui::screens::{form, summary};

impl App {
    pub fn draw(&self, area: Rect, buffer: &mut Buffer) {
        let asking = self.asking().map(|asking| asking.line(area.width as usize));
        let body = frame::render(
            self.look,
            self.nav.at(),
            &self.view,
            frame::Hints {
                editing: self.editing.is_some(),
                kills: self.kill_target().is_some(),
                changes: self
                    .changes_offered()
                    .map_or(&[], vigil_model::AccountObject::changings),
                typing: self.typing(),
                asking: asking.as_deref(),
                level: self.level,
                message: self.message.as_deref(),
                back: self.back(),
                panel: self.detail_showing(self.body.get()) || self.showing_why(),
                choosing: self.choosing(),
                choosing_acts: self
                    .chooser
                    .choosing()
                    .is_some_and(crate::ui::Choosing::asks_before_acting),
                sorts: !self.sortable().is_empty(),
                filters: self.nav.at() == Screen::FINDINGS
                    || !self.pane_kinds().is_empty()
                    || !self.facets_offered().is_empty(),
                marks: self.marking_offered(),
                stops: self.kill_target() == Some(vigil_model::KillTarget::Program),
                buttons: self.button_at().is_some(),
                arranges: self
                    .pane()
                    .and_then(|pane| pane.arrangements().first().map(|one| one.key)),
                to_object: self.nav.at() == Screen::FINDINGS,
            },
            area,
            buffer,
        );
        self.body.set(body);

        if let Some(editing) = &self.editing {
            form::render(editing, self.look, body, buffer);
            return;
        }

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

        if let Some(sheet) = &self.paper {
            paper::render(sheet, self.look, area, buffer);
        }
        if self.helping {
            help::render(self.look, area, buffer);
        }
    }

    pub(in crate::ui::app) fn back(&self) -> Back {
        match (self.nav.at(), self.nav.came_from()) {
            (Screen::HOME, _) => Back::Nowhere,
            (_, Some(_)) => Back::Finding,
            (_, None) => Back::MainScreen,
        }
    }

    pub(in crate::ui::app) fn draw_screen(&self, body: Rect, buffer: &mut Buffer) {
        match self.nav.at() {
            Screen::HOME => self.draw_home(body, buffer),
            Screen::SUMMARY => summary::render(
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
}
