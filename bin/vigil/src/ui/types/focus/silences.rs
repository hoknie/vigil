use std::path::PathBuf;

use vigil_config::{Source, Suppression};

use crate::ui::Motion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Silenced {
    pub file: PathBuf,
    pub suppression: Suppression,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Silences {
    written: Vec<Silenced>,
    refused: Option<String>,
    at: usize,
}

impl Silences {
    pub fn read(sources: Result<Vec<Source>, String>) -> Silences {
        match sources {
            Ok(sources) => Silences {
                written: sources
                    .into_iter()
                    .flat_map(|source| {
                        let file = source.path;
                        source
                            .suppressions
                            .into_iter()
                            .map(move |suppression| Silenced {
                                file: file.clone(),
                                suppression,
                            })
                    })
                    .collect(),
                refused: None,
                at: 0,
            },
            Err(why) => Silences {
                written: Vec::new(),
                refused: Some(why),
                at: 0,
            },
        }
    }

    pub fn written(&self) -> &[Silenced] {
        &self.written
    }

    pub fn refused(&self) -> Option<&str> {
        self.refused.as_deref()
    }

    pub fn at(&self) -> usize {
        self.at
    }

    pub fn read_again(&mut self, again: Silences) {
        let at = self.at;
        *self = again;
        self.at = at;
    }

    pub fn step(&mut self, motion: Motion, page: usize, total: usize) {
        let last = total.saturating_sub(1);
        self.at = match (motion.distance(page.max(1)), motion) {
            (Some(by), _) => self.at.saturating_add_signed(by).min(last),
            (None, Motion::First) => 0,
            (None, _) => last,
        };
    }

    pub fn settle(&mut self, total: usize) {
        self.at = self.at.min(total.saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(path: &str, keys: &[&str]) -> Source {
        Source {
            path: PathBuf::from(path),
            suppressions: keys
                .iter()
                .map(|key| Suppression {
                    finding_key: Some((*key).to_string()),
                    reason: "expected here".into(),
                    ..Suppression::default()
                })
                .collect(),
        }
    }

    #[test]
    fn every_entry_of_every_file_is_one_row_and_carries_the_file_it_came_from() {
        let silences = Silences::read(Ok(vec![
            source("/etc/vigil/vigil.yaml", &[]),
            source("/etc/vigil/suppressions/10-deploy.yaml", &["a|b", "c|d"]),
            source("/etc/vigil/suppressions/console.yaml", &["e|f"]),
        ]));

        let files: Vec<String> = silences
            .written()
            .iter()
            .map(|row| row.file.display().to_string())
            .collect();
        assert_eq!(
            files,
            [
                "/etc/vigil/suppressions/10-deploy.yaml",
                "/etc/vigil/suppressions/10-deploy.yaml",
                "/etc/vigil/suppressions/console.yaml",
            ]
        );
    }

    #[test]
    fn a_file_the_console_could_not_read_is_kept_as_the_reason_and_not_as_an_empty_list() {
        let silences = Silences::read(Err("/etc/vigil/vigil.yaml: permission denied".into()));

        assert!(silences.written().is_empty());
        assert_eq!(
            silences.refused(),
            Some("/etc/vigil/vigil.yaml: permission denied"),
            "an unreadable file drawn as nothing silenced would tell the reader the host is loud"
        );
    }

    #[test]
    fn the_cursor_stays_on_the_list_whatever_it_is_asked_to_do() {
        let mut silences = Silences::default();

        silences.step(Motion::Down, 10, 3);
        silences.step(Motion::PageDown, 10, 3);
        assert_eq!(silences.at(), 2);
        silences.step(Motion::First, 10, 3);
        assert_eq!(silences.at(), 0);
        silences.step(Motion::Up, 10, 3);
        assert_eq!(silences.at(), 0);
        silences.step(Motion::Last, 10, 3);
        silences.settle(1);
        assert_eq!(
            silences.at(),
            0,
            "a row taken out leaves the cursor on one that is still there"
        );
    }

    #[test]
    fn reading_the_files_again_keeps_the_cursor_where_the_reader_left_it() {
        let mut silences = Silences::read(Ok(vec![source("/x.yaml", &["a|b", "c|d", "e|f"])]));
        silences.step(Motion::Down, 10, 3);

        silences.read_again(Silences::read(Ok(vec![source("/x.yaml", &["a|b", "e|f"])])));

        assert_eq!(silences.at(), 1);
        assert_eq!(silences.written().len(), 2);
    }
}
