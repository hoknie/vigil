use serde_json::Value;
use vigil_model::Snapshot;

use super::engine::Engine;
use super::subject::Subject;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    pub watched: bool,
    pub dump_read: bool,
    pub present: bool,
    pub unanswered: Vec<Subject>,
}

impl Standing {
    pub fn of(engine_row: Option<&Value>) -> Standing {
        let Some(row) = engine_row else {
            return Standing {
                watched: false,
                dump_read: false,
                present: false,
                unanswered: Vec::new(),
            };
        };
        let flag = |field: &str| row.get(field).and_then(Value::as_bool).unwrap_or(false);

        Standing {
            watched: true,
            dump_read: flag("dump_read"),
            present: flag("present"),
            unanswered: row
                .get("unanswered")
                .and_then(Value::as_array)
                .map(|named| {
                    named
                        .iter()
                        .filter_map(Value::as_str)
                        .filter_map(Subject::named)
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn in_reading(reading: &Snapshot, engine: Engine) -> Standing {
        let key = format!(
            "{}|{}|{}",
            engine.name(),
            Subject::Engine.as_str(),
            engine.name()
        );
        Standing::of(reading.items.get(&key))
    }

    pub fn answers(&self) -> bool {
        self.watched && self.dump_read && self.present
    }

    pub fn silent_on(&self, subject: Subject) -> bool {
        match subject {
            _ if !self.watched => true,
            Subject::Engine => false,
            Subject::Registry => self.unanswered.contains(&Subject::Registry),
            _ if !self.dump_read || !self.present => true,
            Subject::Project => {
                self.unanswered.contains(&Subject::Project)
                    || self.unanswered.contains(&Subject::Container)
            }
            _ => self.unanswered.contains(&subject),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn engine(present: bool, unanswered: &[&str]) -> Value {
        json!({"subject": "engine", "present": present, "dump_read": true, "unanswered": unanswered})
    }

    #[test]
    fn an_engine_that_answered_everything_is_silent_on_nothing() {
        let standing = Standing::of(Some(&engine(true, &[])));

        assert!(standing.answers());
        for subject in Subject::ALL {
            assert!(!standing.silent_on(subject), "{subject:?}");
        }
    }

    #[test]
    fn a_failed_list_of_containers_leaves_the_projects_unknown_too() {
        let standing = Standing::of(Some(&engine(true, &["container"])));

        assert!(standing.silent_on(Subject::Container));
        assert!(
            standing.silent_on(Subject::Project),
            "a compose project is read off the labels of the containers, so a list of \
             containers that did not arrive is a list of projects that did not arrive"
        );
        assert!(!standing.silent_on(Subject::Image));
    }

    #[test]
    fn an_engine_not_installed_or_not_read_says_nothing_about_what_it_holds_but_its_files_do() {
        for standing in [
            Standing::of(Some(&engine(false, &[]))),
            Standing::of(Some(
                &json!({"subject": "engine", "present": false, "dump_read": false, "unanswered": []}),
            )),
        ] {
            assert!(!standing.answers());
            assert!(standing.silent_on(Subject::Image));
            assert!(standing.silent_on(Subject::Secret));
            assert!(
                !standing.silent_on(Subject::Registry),
                "the registries are read from the engine's files under /etc, which are there \
                 whether or not the engine answered"
            );
        }
    }

    #[test]
    fn an_engine_nobody_asked_about_is_silent_on_everything() {
        let standing = Standing::of(None);

        assert!(!standing.watched);
        for subject in Subject::ALL {
            assert!(standing.silent_on(subject), "{subject:?}");
        }
    }
}
