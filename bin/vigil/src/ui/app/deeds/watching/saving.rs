use vigil_config::Watch;
use vigil_files::{asked_for, path_watched};

use super::super::accounts::sentence;
use super::changing::NOT_READ_YET;
use crate::config;
use crate::ui::app::App;
use crate::ui::{Paper, Reading};

const WROTE_NOTHING: &str = "THE CONFIGURATION FILE WAS NOT TOUCHED";

const WROTE: &str = "THE CONFIGURATION FILE WAS WRITTEN";

impl App {
    pub(in crate::ui::app) fn save_the_watched_path(&mut self) {
        let asked = match self.what_the_watching_form_asks_for() {
            Ok(asked) => asked,
            Err(why) => {
                self.say_while_watching(sentence(&why));
                return;
            }
        };
        match config::watch(&self.configuration_path(), &asked) {
            Err(why) => self.say_while_watching(sentence(&why)),
            Ok(done) => {
                self.editing = None;
                self.paper = Some(Paper::of(
                    match done.entries {
                        0 => WROTE_NOTHING,
                        _ => WROTE,
                    },
                    done.said,
                ));
                self.refresh_wanted = true;
            }
        }
    }

    pub(in crate::ui::app) fn what_the_watching_form_asks_for(&self) -> Result<Watch, String> {
        let editing = self.editing.as_ref().ok_or("no form is open")?;
        let section = self
            .section_of(editing.screen())
            .ok_or("the section this form was opened on is no longer drawn")?;
        let panes = section.panes();
        let pane = panes
            .get(editing.pane())
            .ok_or("the list this form was opened on is no longer drawn")?;
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return Err(NOT_READ_YET.to_string());
        };

        if let Some(key) = editing.row() {
            if !snapshot.items.contains_key(key) {
                return Err(format!(
                    "{key} is no longer in the reading: it changed or went away since this form \
                     was opened, and nothing was written"
                ));
            }
            path_watched(key).ok_or("this row is not a path the configuration file names")?;
        }

        let watched = asked_for(editing.form())?;
        Ok(Watch::of(watched.path(), watched.ceiling_bytes()))
    }

    pub(in crate::ui::app) fn say_while_watching(&mut self, said: String) {
        if let Some(editing) = self.editing.as_mut() {
            editing.said(said);
        }
    }
}
