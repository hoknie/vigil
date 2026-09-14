use vigil_model::Finding;

use crate::ui::app::App;

use crate::config;
use crate::ui::screens::findings;
use crate::ui::{Asking, Deed, Level, Motion, Offset, Screen};

const PICK_HERE: char = 'x';

const PICK_EVERY_ROW: char = 'a';

const BRING_BACK: char = 'u';

const NOTHING_TAKEN_OFF: &str = "Nothing has been silenced from this console.";

impl App {
    pub(in crate::ui::app) fn picked_here(&self) -> bool {
        self.nav.at() == Screen::FINDINGS && !self.picked.is_empty()
    }

    pub(in crate::ui::app) fn asking(&self) -> Option<&Asking> {
        self.asking.as_ref()
    }

    pub(in crate::ui::app) fn configuration_path(&self) -> String {
        if let Some(named) = &self.named_configuration {
            return named.clone();
        }
        self.view
            .status
            .as_ref()
            .and_then(|status| status.agent.configuration_path.clone())
            .unwrap_or_else(|| config::DEFAULT_PATH.to_string())
    }

    pub(in crate::ui::app) fn pick_along(&mut self, motion: Motion) {
        if !self.on_the_findings() {
            self.move_within(motion);
            return;
        }
        let keys = findings::keys(&self.passing());
        self.picked.opening(&keys, self.nav.findings.at());
        self.step(&keys, motion);
        self.picked.reaching(&keys, self.nav.findings.at());
    }

    pub(in crate::ui::app) fn gather_along(&mut self, motion: Motion) {
        if !self.on_the_findings() {
            self.move_within(motion);
            return;
        }
        let keys = findings::keys(&self.passing());
        if let Some(key) = keys.get(self.nav.findings.at()) {
            self.picked.toggle(key);
        }
        self.step(&keys, motion);
    }

    fn on_the_findings(&self) -> bool {
        self.nav.at() == Screen::FINDINGS && self.level == Level::List
    }

    pub(in crate::ui::app) fn picking_key(&mut self, key: char) -> bool {
        let keys = findings::keys(&self.passing());
        match key {
            PICK_HERE => match keys.get(self.nav.findings.at()) {
                Some(row) => {
                    self.picked.toggle(row);
                    true
                }
                None => false,
            },
            PICK_EVERY_ROW => {
                self.picked.every(&keys);
                true
            }
            BRING_BACK => self.bring_back(),
            key => match Deed::of(key) {
                Some(deed) => self.ask_about(deed),
                None => false,
            },
        }
    }

    fn ask_about(&mut self, deed: Deed) -> bool {
        let reached = self.reached();
        match Asking::about(deed, &reached) {
            Some(asking) => {
                self.asking = Some(asking);
                true
            }
            None => false,
        }
    }

    pub(in crate::ui::app) fn typed_into_the_reason(&mut self, character: char) {
        if let Some(asking) = self.asking.as_mut() {
            asking.type_character(character);
        }
    }

    pub(in crate::ui::app) fn erased_from_the_reason(&mut self) {
        if let Some(asking) = self.asking.as_mut() {
            asking.erase();
        }
    }

    pub(in crate::ui::app) fn left_the_reason(&mut self) {
        self.asking = None;
    }

    pub(in crate::ui::app) fn do_the_deed(&mut self) {
        let Some(asking) = self.asking.take() else {
            return;
        };
        if asking.reason().is_empty() {
            self.asking = Some(asking);
            self.message = Some(
                "A suppression needs a reason: the daemon refuses one without it, and a year \
                 from now it is the only thing that says why this host is quiet about it."
                    .to_string(),
            );
            return;
        }

        let asked = config::Options {
            keys: asking.keys().to_vec(),
            reason: asking.reason().to_string(),
            path: self.configuration_path(),
            ..config::Options::default()
        };
        match config::add(&asked) {
            Err(why) => self.message = Some(format!("Nothing was silenced. {why}")),
            Ok(done) => {
                match asking.deed() {
                    Deed::Remove => self.dismissed.silence(asking.keys().to_vec()),
                }
                self.picked.clear();
                self.message = Some(match done.entries {
                    0 => format!(
                        "{} object(s) already silenced in {}",
                        asking.keys().len(),
                        self.configuration_path()
                    ),
                    written => format!(
                        "{written} object(s) silenced \u{b7} {} applies it \u{b7} written to {}",
                        config::RESTART,
                        self.configuration_path()
                    ),
                });
            }
        }
    }

    fn bring_back(&mut self) -> bool {
        if self.dismissed.is_empty() {
            self.message = Some(NOTHING_TAKEN_OFF.to_string());
            return false;
        }
        let written = self.dismissed.objects();
        let count = written.len();
        let taken_out = config::remove(&config::Options {
            keys: written,
            path: self.configuration_path(),
            ..config::Options::default()
        });

        match taken_out {
            Err(why) => {
                self.message = Some(format!(
                    "The rows are back on this screen, and {} still holds the entry: {why}",
                    self.configuration_path()
                ));
                self.dismissed.bring_back();
                true
            }
            Ok(_) => {
                self.dismissed.bring_back();
                self.message = Some(format!(
                    "{count} object(s) no longer silenced \u{b7} {} applies it",
                    config::RESTART
                ));
                true
            }
        }
    }

    fn reached(&self) -> Vec<&Finding> {
        let passing = self.passing();
        if !self.picked.is_empty() {
            return passing
                .into_iter()
                .filter(|finding| self.picked.holds(&finding.event_id))
                .collect();
        }
        passing
            .get(self.nav.findings.at())
            .copied()
            .into_iter()
            .collect()
    }

    fn step(&mut self, keys: &[String], motion: Motion) {
        let rows = (self.body.get().height as usize).saturating_sub(2);
        self.nav.findings.step(motion, keys, rows);
        self.nav.difference = Offset::default();
    }
}
