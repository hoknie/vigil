use super::App;

use crate::config;
use crate::ui::screens::silences::{Shown, shown};
use crate::ui::{Level, Motion, Offset, Screen, Silences};

pub const LISTS: [&str; 2] = ["reported", "silenced"];

const REPORT_AGAIN: char = 'u';

const NOTHING_UNDER_THE_CURSOR: &str = "There is no entry under the cursor to take out.";

pub const IN_THE_ORDER_OF_THE_FILES: &str = "The silenced list is in the order of its files and \
                                             holds every entry: nothing here sorts, narrows or \
                                             searches.";

impl App {
    pub(in crate::ui::app) fn on_the_silenced(&self) -> bool {
        self.nav.at() == Screen::FINDINGS && self.silences.is_some()
    }

    pub(in crate::ui::app) fn findings_list(&self) -> usize {
        usize::from(self.silences.is_some())
    }

    pub(in crate::ui::app) fn choose_the_findings_list(&mut self, at: usize) {
        match at.min(LISTS.len() - 1) {
            0 => self.silences = None,
            _ if self.silences.is_none() => self.silences = Some(self.silences_read()),
            _ => {}
        }
        self.picked.clear();
        self.detail_open = false;
        self.nav.difference = Offset::default();
        self.refresh_wanted = true;
    }

    pub(in crate::ui::app) fn step_the_findings_lists(&mut self, by: isize) {
        let count = LISTS.len() as isize;
        let at = (self.findings_list() as isize + by).rem_euclid(count) as usize;
        self.choose_the_findings_list(at);
    }

    fn silences_read(&self) -> Silences {
        Silences::read(config::every(&self.configuration_path()))
    }

    pub(in crate::ui::app) fn silences_running(&self) -> Option<&[String]> {
        self.view
            .status
            .as_ref()
            .map(|status| status.agent.silence.suppressions.as_slice())
    }

    fn silences_shown(&self) -> Vec<Shown> {
        match &self.silences {
            Some(silences) => shown(silences, self.silences_running()),
            None => Vec::new(),
        }
    }

    pub(in crate::ui::app) fn silenced_at_the_top(&self) -> bool {
        self.silences
            .as_ref()
            .is_none_or(|silences| silences.at() == 0)
    }

    pub(in crate::ui::app) fn move_along_the_silenced(&mut self, motion: Motion) {
        let total = self.silences_shown().len();
        let page = (self.body.get().height as usize).saturating_sub(3);
        if let Some(silences) = self.silences.as_mut() {
            silences.step(motion, page, total);
        }
    }

    pub(in crate::ui::app) fn silenced_letter(&mut self, key: char) -> bool {
        match key {
            REPORT_AGAIN => {
                self.report_it_again();
                true
            }
            _ => false,
        }
    }

    pub(in crate::ui::app) fn point_at_the_silenced(&mut self, at: usize) {
        let total = self.silences_shown().len();
        if let Some(silences) = self.silences.as_mut() {
            silences.step(Motion::First, 1, total);
            for _ in 0..at {
                silences.step(Motion::Down, 1, total);
            }
        }
        self.level = Level::List;
    }

    pub(in crate::ui::app) fn read_the_silences_again(&mut self) {
        if self.silences.is_none() {
            return;
        }
        let again = self.silences_read();
        let total = shown(&again, self.silences_running()).len();
        if let Some(silences) = self.silences.as_mut() {
            silences.read_again(again);
            silences.settle(total);
        }
    }

    fn report_it_again(&mut self) {
        let Some(silences) = &self.silences else {
            return;
        };
        let rows = self.silences_shown();
        let Some(row) = rows.get(silences.at()) else {
            self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
            return;
        };
        let Some(silenced) = row
            .written
            .and_then(|at| silences.written().get(at))
            .cloned()
        else {
            self.message = Some(
                "This one is already out of its file; the agent lets it go on its next round, \
                 or says in its log why a file could not be read."
                    .to_string(),
            );
            return;
        };

        let asked = config::Options {
            path: self.configuration_path(),
            ..config::Options::default()
        };
        self.message = Some(
            match config::take_out(&asked, &silenced.file, &silenced.suppression) {
                Err(why) => format!("Nothing was taken out. {why}"),
                Ok(done) if done.entries == 0 => done.said.join(" "),
                Ok(_) => {
                    if let Some(key) = &silenced.suppression.finding_key {
                        self.dismissed.forget(key);
                    }
                    format!(
                        "{} is reported again \u{b7} {} \u{b7} taken out of {}",
                        row.object,
                        config::NEXT_ROUND,
                        silenced.file.display()
                    )
                }
            },
        );
        self.read_the_silences_again();
    }
}
