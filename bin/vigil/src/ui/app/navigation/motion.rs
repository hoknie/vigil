use crate::ui::app::App;

use crate::ui::screens::unknown::unknown_readings;
use crate::ui::screens::{findings, home, summary};
use crate::ui::{Level, Motion, Offset, Screen};

impl App {
    pub(in crate::ui::app) fn move_within(&mut self, motion: Motion) {
        let body = self.body.get();
        let width = self.look.text_width(body.width);
        let rows = (body.height as usize).saturating_sub(2);
        let page = body.height as usize;

        match self.level {
            Level::Menu => self.along_the_row(motion),
            Level::Detail if self.climbing_out_of_the_detail(motion) => self.leave_the_row(),
            Level::Detail => {
                if let Some(area) = self.detail_area() {
                    let total = self.detail_height(area);
                    self.nav
                        .difference
                        .step(motion, total, area.height as usize);
                }
            }
            Level::List if self.climbing(motion) => self.leave_the_row(),
            Level::List => match self.nav.at() {
                Screen::HOME => {
                    let keys = home::keys(&self.view);
                    self.nav.sections.step(motion, &keys, rows);
                }
                Screen::SUMMARY => {
                    let total = summary::height(
                        &self.view,
                        self.look,
                        width,
                        self.saying(),
                        self.gone.as_ref(),
                    );
                    self.nav.summary.step(motion, total, page);
                }
                screen if screen.draws_a_reading() => {
                    let keys = self.pane_listed();
                    if let Some(panes) = self.panes_mut() {
                        panes
                            .cursor_mut()
                            .step(motion, keys.as_ref(), rows.saturating_sub(1));
                    }
                    self.nav.difference = Offset::default();
                    self.rest_the_buttons();
                }
                Screen::FINDINGS => {
                    let keys = findings::keys(&self.passing());
                    self.nav.findings.step(motion, &keys, rows);
                    self.nav.difference = Offset::default();
                }
                _ => {}
            },
        }
    }

    fn climbing_out_of_the_detail(&self, motion: Motion) -> bool {
        matches!(motion, Motion::Up | Motion::PageUp) && self.nav.difference.top() == 0
    }

    fn climbing(&self, motion: Motion) -> bool {
        matches!(motion, Motion::Up | Motion::PageUp)
            && self.nav.at() != Screen::HOME
            && self.at_the_top()
    }

    fn at_the_top(&self) -> bool {
        match self.nav.at() {
            Screen::HOME => self.nav.sections.at() == 0,
            Screen::SUMMARY => self.nav.summary.top() == 0,
            Screen::FINDINGS => self.nav.findings.at() == 0,
            screen if screen.draws_a_reading() => self.panes().is_some_and(|panes| panes.at() == 0),
            _ => true,
        }
    }

    fn along_the_row(&mut self, motion: Motion) {
        match motion {
            Motion::Down | Motion::PageDown => self.level = Level::List,
            Motion::Up | Motion::PageUp => self.leave_the_row(),
            Motion::First | Motion::Last => {
                let ends = |count: usize| match motion {
                    Motion::First => 0,
                    _ => count.saturating_sub(1),
                };
                match self.nav.at() {
                    screen if screen.draws_a_reading() => {
                        let shown = self.shown_panes();
                        let at = ends(shown.len());
                        if let Some(index) = shown.get(at).copied()
                            && let Some(panes) = self.panes_mut()
                        {
                            panes.show(index);
                        }
                    }
                    _ => {}
                }
                self.detail_open = false;
                self.refresh_wanted = true;
            }
        }
    }

    pub(in crate::ui::app) fn settle(&mut self) {
        let body = self.body.get();
        let width = self.look.text_width(body.width);
        let page = body.height as usize;

        self.nav.sections.settle(&home::keys(&self.view));
        let unknown = unknown_readings(&self.view).len();
        self.nav.lists.ready(Screen::UNKNOWN.name(), unknown);
        let rows = self.pane_listed();
        let gone = self.marks_gone_from_the_reading();
        if let Some(panes) = self.panes_mut() {
            panes.cursor_mut().settle(rows.as_ref());
            panes.forget_marks(&gone);
        }
        self.dismissed.settle(&self.view.found.findings);
        let findings = findings::keys(&self.passing());
        self.nav.findings.settle(&findings);
        self.picked.settle(&findings);
        self.nav.summary.settle(
            summary::height(
                &self.view,
                self.look,
                width,
                self.saying(),
                self.gone.as_ref(),
            ),
            page,
        );
        if let Some(area) = self.detail_area() {
            let total = self.detail_height(area);
            self.nav.difference.settle(total, area.height as usize);
        }
        if self.level == Level::Detail && !self.has_detail() {
            self.level = Level::List;
            self.detail_open = false;
        }
    }
}
