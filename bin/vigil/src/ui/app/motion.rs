use super::App;

use crate::ui::screens::{findings, home, ports, summary};
use crate::ui::{Level, Motion, Offset, Program, Screen, Startup, Subject};

impl App {
    pub(super) fn move_within(&mut self, motion: Motion) {
        let body = self.body.get();
        let width = self.look.text_width(body.width);
        let rows = (body.height as usize).saturating_sub(2);
        let page = body.height as usize;

        match self.level {
            Level::Menu => self.along_the_row(motion),
            Level::Detail => {
                if let Some(area) = self.detail_area() {
                    let total = self.detail_height(area);
                    self.nav
                        .difference
                        .step(motion, total, area.height as usize);
                }
            }
            Level::List => match self.nav.at() {
                Screen::Home => {
                    let keys = home::keys(&self.view);
                    self.nav.sections.step(motion, &keys, rows);
                }
                Screen::Summary => {
                    let total = summary::height(&self.view, self.look, width);
                    self.nav.summary.step(motion, total, page);
                }
                Screen::Accounts => {
                    let keys = self.accounts_keys();
                    self.nav
                        .lists
                        .accounts
                        .cursor_mut()
                        .step(motion, &keys, rows);
                    self.nav.difference = Offset::default();
                }
                Screen::Ports => {
                    let keys = self.ports_keys();
                    self.nav
                        .lists
                        .ports
                        .cursor_mut()
                        .step(motion, &keys, rows.saturating_sub(1));
                    self.nav.difference = Offset::default();
                }
                Screen::Programs => {
                    let keys = self.programs_keys();
                    self.nav
                        .lists
                        .programs
                        .cursor_mut()
                        .step(motion, &keys, rows);
                    self.nav.difference = Offset::default();
                }
                Screen::Startup => {
                    let keys = self.startup_keys();
                    self.nav
                        .lists
                        .startup
                        .cursor_mut()
                        .step(motion, &keys, rows);
                    self.nav.difference = Offset::default();
                }
                Screen::Findings => {
                    let keys = findings::keys(&self.filter.passing(&self.view.found.findings));
                    self.nav.findings.step(motion, &keys, rows);
                    self.nav.difference = Offset::default();
                }
            },
        }
    }

    fn along_the_row(&mut self, motion: Motion) {
        match motion {
            Motion::Down | Motion::PageDown => self.level = Level::List,
            Motion::Up | Motion::PageUp => self.go_back(),
            Motion::First | Motion::Last => {
                let ends = |count: usize| match motion {
                    Motion::First => 0,
                    _ => count.saturating_sub(1),
                };
                match self.nav.at() {
                    Screen::Ports => {
                        let names = ports::Arrangement::ALL;
                        self.nav.lists.ports.show(names[ends(names.len())]);
                    }
                    Screen::Programs => {
                        let names = Program::ALL;
                        self.nav.lists.programs.show(names[ends(names.len())]);
                    }
                    Screen::Startup => {
                        let shown = Startup::on(&self.view);
                        if let Some(list) = shown.get(ends(shown.len())).copied() {
                            self.nav.lists.startup.show(list);
                        }
                    }
                    _ => {
                        let shown = Subject::on(&self.view);
                        if let Some(subject) = shown.get(ends(shown.len())).copied() {
                            self.nav.lists.accounts.show(subject);
                        }
                    }
                }
                self.detail_open = false;
                self.refresh_wanted = true;
            }
        }
    }

    pub(super) fn settle(&mut self) {
        let body = self.body.get();
        let width = self.look.text_width(body.width);
        let page = body.height as usize;

        self.nav.sections.settle(&home::keys(&self.view));
        let listening = self.ports_keys();
        self.nav.lists.ports.cursor_mut().settle(&listening);
        let findings = findings::keys(&self.filter.passing(&self.view.found.findings));
        self.nav.findings.settle(&findings);
        self.nav
            .summary
            .settle(summary::height(&self.view, self.look, width), page);
        let accounts = self.accounts_keys();
        self.nav.lists.accounts.cursor_mut().settle(&accounts);
        let running = self.programs_keys();
        self.nav.lists.programs.cursor_mut().settle(&running);
        let starting = self.startup_keys();
        self.nav.lists.startup.cursor_mut().settle(&starting);
        if let Some(area) = self.detail_area() {
            let total = self.detail_height(area);
            self.nav.difference.settle(total, area.height as usize);
        }
        if self.level == Level::Detail && !self.has_detail() {
            self.level = Level::List;
            self.detail_open = false;
        }
        if !Startup::on(&self.view).contains(&self.nav.lists.startup.showing()) {
            self.nav.lists.startup.show(Startup::default());
        }
    }
}
