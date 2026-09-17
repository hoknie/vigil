use vigil_files::path_watched;

use super::super::accounts::{DELETE, sentence};
use crate::config;
use crate::ui::app::App;
use crate::ui::{Choosing, Paper};

const STOPPED: &str = "THE CONFIGURATION FILE WAS WRITTEN";

const LEFT: &str = "THE CONFIGURATION FILE WAS NOT TOUCHED";

const NOT_NAMED_HERE: &str = "This path is watched by the agent itself and is named in no \
                              configuration file, so there is nothing here to stop.";

impl App {
    pub(super) fn ask_to_stop_watching(&mut self) {
        let Some(path) = self.path_under_the_cursor() else {
            return;
        };

        self.chooser.open_by_key(
            Choosing::Unwatch,
            vec![(DELETE, format!("stop watching {path}"))],
        );
    }

    pub(in crate::ui::app) fn chose_to_stop_watching(&mut self) {
        let Some(path) = self.path_under_the_cursor() else {
            return;
        };

        match config::unwatch(&self.configuration_path(), &path) {
            Err(why) => self.message = Some(sentence(&why)),
            Ok(done) => {
                self.paper = Some(Paper::of(
                    match done.entries {
                        0 => LEFT,
                        _ => STOPPED,
                    },
                    done.said,
                ));
                self.refresh_wanted = true;
            }
        }
    }

    fn path_under_the_cursor(&mut self) -> Option<String> {
        let row = match self.pane_row_under_the_cursor() {
            Some(row) if row.of_the_reading => row,
            _ => {
                self.message =
                    Some("There is no watched path under the cursor to stop.".to_string());
                return None;
            }
        };

        match path_watched(&row.key) {
            Some(path) => Some(path.to_string()),
            None => {
                self.message = Some(NOT_NAMED_HERE.to_string());
                None
            }
        }
    }
}
