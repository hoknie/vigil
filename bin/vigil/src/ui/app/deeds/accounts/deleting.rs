use vigil_model::{AccountChange, Changing};
use vigil_view::RowKey;

use super::asked::Asked;
use super::changing::{DELETE, NOT_READ_YET};
use super::sentence::sentence;
use crate::ui::app::App;
use crate::ui::{Choosing, Paper, Reading};

impl App {
    pub(super) fn deleting(&mut self) {
        let Some(object) = self.offered_here(Changing::Delete) else {
            return;
        };
        let keys = self.what_a_kill_would_take();
        if keys.is_empty() {
            self.message = Some(format!(
                "There is nothing here to delete: no row is marked, and the cursor is not on a {}.",
                object.named()
            ));
            return;
        }
        let (changes, left) = self.deletions(&keys);
        if changes.is_empty() {
            self.message = left
                .first()
                .map(|(key, why)| sentence(&format!("{key}: {why}")));
            return;
        }

        self.chooser.open_by_key(
            Choosing::Delete(object),
            vec![(DELETE, object.deleting().to_string())],
        );
    }

    pub(in crate::ui::app) fn deletions(
        &self,
        keys: &[String],
    ) -> (Vec<AccountChange>, Vec<(String, String)>) {
        let Some(pane) = self.pane() else {
            return (Vec::new(), Vec::new());
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return (
                Vec::new(),
                keys.iter()
                    .map(|key| (key.clone(), NOT_READ_YET.to_string()))
                    .collect(),
            );
        };

        let mut changes = Vec::new();
        let mut left = Vec::new();
        for key in keys {
            if !snapshot.items.contains_key(key) {
                left.push((
                    key.clone(),
                    "no longer in the reading: it changed or went away since it was marked"
                        .to_string(),
                ));
                continue;
            }
            match pane.change(
                snapshot,
                Some(&RowKey::of(key.clone())),
                Changing::Delete,
                None,
            ) {
                Ok(change) => changes.push(change),
                Err(why) => left.push((key.clone(), why)),
            }
        }
        (changes, left)
    }

    pub(in crate::ui::app) fn chose_to_delete(&mut self) {
        let keys = self.what_a_kill_would_take();
        let (changes, left) = self.deletions(&keys);
        if changes.is_empty() {
            self.message = Some(
                "Nothing was asked for: every row that was to be deleted is gone from the \
                 reading or cannot be deleted."
                    .to_string(),
            );
            return;
        }

        match self.ask_for(changes) {
            Asked::Report(report) => self.took_the_changes(&report, &left),
            Asked::Refused { message, advice } => {
                let mut lines = vec![message];
                if let Some(advice) = advice {
                    lines.push(String::new());
                    lines.push(advice);
                }
                self.paper = Some(Paper::of("THE AGENT DID NOTHING", lines));
            }
            Asked::Trouble(said) => self.message = Some(said),
        }
        self.refresh_wanted = true;
    }
}
