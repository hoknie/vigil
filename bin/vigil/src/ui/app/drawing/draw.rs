use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::app::App;
use crate::ui::chrome::chooser;
use crate::ui::chrome::frame;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::chrome::help;
use crate::ui::chrome::paper;
use crate::ui::helpers::words::unreachable;
use crate::ui::screens::{form, graph, history, summary};
use crate::ui::{Aim, Screen, Target};

impl App {
    pub fn draw(&self, area: Rect, buffer: &mut Buffer) {
        self.cursor.set(None);
        self.pointer.drawing();
        self.draw_the_page(area, buffer);
        self.pointer.finished();
    }

    fn draw_the_page(&self, area: Rect, buffer: &mut Buffer) {
        let asking = self.asking().map(|asking| asking.line(area.width as usize));
        let body = frame::render(
            self.look,
            self.nav.at(),
            &self.view,
            frame::Hints {
                editing: self.editing.is_some(),
                listing: self
                    .editing
                    .as_ref()
                    .is_some_and(|editing| editing.dropdown().is_some()),
                history: self.history.is_some(),
                histories: self.pane().is_some_and(|pane| pane.offers().history),
                graph: self.graph.is_some(),
                graphs: self.pane().is_some_and(|pane| pane.offers().graph),
                kills: self.kill_target().is_some(),
                controls: self.control_target(),
                changes: match self.changes_offered() {
                    Some(object) => object.changings(),
                    None => self.written_to_the_configuration(),
                },
                typing: self.typing(),
                asking: asking.as_deref(),
                level: self.level,
                message: self.message.as_deref(),
                back: self.back(),
                panel: self.detail_showing(self.body.get()) || self.showing_why(),
                panes: self.framed_panes(area),
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
                mouse: self.look.interactive().then(|| self.mouse_said()),
            },
            area,
            buffer,
        );
        self.body.set(body);

        if let Some(editing) = &self.editing {
            let (cursor, aims) = form::render(editing, self.look, body, buffer);
            self.cursor.set(cursor);
            self.aimed(aims);
            return;
        }
        if let Some(opened) = &self.graph {
            self.pointer.put(body, Target::Detail);
            for (drawn, target) in graph::render(
                &self.graph_pieces(),
                opened.watching(),
                self.look,
                opened.top(),
                body,
                buffer,
            ) {
                self.pointer.put(drawn, target);
            }
            return;
        }
        if let Some(opened) = &self.history {
            self.pointer.put(body, Target::Detail);
            for (drawn, target) in history::render(opened, self.look, body, buffer) {
                self.pointer.put(drawn, target);
            }
            return;
        }

        if !self.view.has_reading() {
            unreachable::render(&self.view, self.look, body, buffer);
        } else {
            self.draw_screen(body, buffer);
        }
        let aims = chooser::render(&self.chooser, self.look, body, buffer);
        self.aimed(aims);

        if let Some(sheet) = &self.paper {
            paper::render(sheet, self.look, area, buffer);
        }
        if self.helping {
            help::render(self.look, area, buffer);
        }
    }

    fn aimed(&self, aims: Vec<(Aim, Rect)>) {
        for (aim, drawn) in aims {
            self.pointer.put(drawn, Target::Aim(aim));
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
            Screen::SUMMARY => {
                self.pointer.put(body, Target::List);
                summary::render(
                    &self.view,
                    self.look,
                    self.nav.summary.top(),
                    self.saying(),
                    self.gone.as_ref(),
                    body,
                    buffer,
                )
            }
            _ => self.draw_listed(body, buffer),
        }
    }
}
