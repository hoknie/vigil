use vigil_model::Finding;

use crate::ui::app::App;

use crate::config;
use crate::ui::screens::findings;
use crate::ui::{Asking, Choosing, Deed, Level, Motion, Offset, Screen};

const PICK_HERE: char = 'x';

const PICK_EVERY_ROW: char = 'a';

const BRING_BACK: char = 'u';

const NOTHING_TAKEN_OFF: &str = "Nothing has been silenced from this console.";

const THIS_OBJECT: char = 'o';

const KIND_FROM_PROGRAM: char = 'k';

const EVERYTHING_FROM_PROGRAM: char = 'p';

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
                Some(deed) if self.every_reached_has_a_program() => self.offer_how_to_silence(deed),
                Some(deed) => self.ask_about(deed),
                None => false,
            },
        }
    }

    fn every_reached_has_a_program(&self) -> bool {
        let reached = self.reached();
        !reached.is_empty() && reached.iter().all(|finding| finding.program().is_some())
    }

    fn offer_how_to_silence(&mut self, _deed: Deed) -> bool {
        let reached = self.reached();
        let (kind, program) = match reached.as_slice() {
            [one] => (
                one.kind.as_str().to_string(),
                one.program().unwrap_or_default().to_string(),
            ),
            _ => ("each kind".to_string(), "its program".to_string()),
        };
        self.chooser.open_by_key(
            Choosing::Silence,
            vec![
                (THIS_OBJECT, "this object".to_string()),
                (KIND_FROM_PROGRAM, format!("{kind} from {program}")),
                (
                    EVERYTHING_FROM_PROGRAM,
                    format!("everything from {program}"),
                ),
            ],
        );
        true
    }

    pub(in crate::ui::app) fn chose_how_to_silence(&mut self, at: usize) {
        let reached = self.reached();
        self.asking = match at {
            0 => Asking::about(Deed::Remove, &reached),
            1 => Asking::about_programs(Deed::Remove, &reached, true),
            _ => Asking::about_programs(Deed::Remove, &reached, false),
        };
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

        if !asking.programs().is_empty() {
            self.silence_programs(&asking);
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
                        "{written} object(s) silenced \u{b7} {} \u{b7} written to {}",
                        config::NEXT_ROUND,
                        done.file
                            .as_ref()
                            .map(|file| file.display().to_string())
                            .unwrap_or_else(|| self.configuration_path())
                    ),
                });
            }
        }
    }

    fn silence_programs(&mut self, asking: &Asking) {
        let mut written = 0;
        let mut file = None;
        let mut entries = Vec::new();
        for (program, kind) in asking.programs() {
            let asked = config::Options {
                program: Some(program.clone()),
                kind: kind.clone(),
                reason: asking.reason().to_string(),
                path: self.configuration_path(),
                ..config::Options::default()
            };
            match config::add(&asked) {
                Err(why) => {
                    self.message = Some(format!("Nothing was silenced. {why}"));
                    return;
                }
                Ok(done) => {
                    written += done.entries;
                    if done.file.is_some() {
                        file = done.file;
                    }
                    entries.push(vigil_config::Suppression {
                        program: Some(program.clone()),
                        kind: kind.clone(),
                        reason: asking.reason().to_string(),
                        ..vigil_config::Suppression::default()
                    });
                }
            }
        }

        let written_to = file
            .as_ref()
            .map(|file| file.display().to_string())
            .unwrap_or_else(|| self.configuration_path());
        if let Some(file) = file {
            self.dismissed.silence_entries(file, entries);
        }
        self.picked.clear();
        self.message = Some(match written {
            0 => format!(
                "{} entry(ies) already silenced in {}",
                asking.programs().len(),
                self.configuration_path()
            ),
            written => format!(
                "{written} entry(ies) written \u{b7} {} \u{b7} written to {written_to}",
                config::NEXT_ROUND
            ),
        });
    }

    fn bring_back(&mut self) -> bool {
        if self.dismissed.is_empty() {
            self.message = Some(NOTHING_TAKEN_OFF.to_string());
            return false;
        }
        for (file, entry) in self.dismissed.entries().to_vec() {
            let asked = config::Options {
                path: self.configuration_path(),
                ..config::Options::default()
            };
            if let Err(why) = config::take_out(&asked, &file, &entry) {
                self.message = Some(format!(
                    "The rows are back on this screen, and {} still holds the entry: {why}",
                    file.display()
                ));
                self.dismissed.bring_back();
                return true;
            }
        }
        let written = self.dismissed.objects();
        let count = written.len() + self.dismissed.entries().len();
        if written.is_empty() {
            self.dismissed.bring_back();
            self.message = Some(format!(
                "{count} entry(ies) no longer silenced \u{b7} {}",
                config::NEXT_ROUND
            ));
            return true;
        }
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
                    "{count} object(s) no longer silenced \u{b7} {}",
                    config::NEXT_ROUND
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
