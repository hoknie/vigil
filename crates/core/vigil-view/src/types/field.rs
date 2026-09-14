use super::choice::Choice;
use super::entry::Entry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub label: String,
    pub entry: Entry,
    pub was: Entry,
    pub hint: Option<String>,
}

impl Field {
    fn of(name: &'static str, label: impl Into<String>, entry: Entry) -> Field {
        Field {
            name,
            label: label.into(),
            was: entry.clone(),
            entry,
            hint: None,
        }
    }

    pub fn fixed(name: &'static str, label: impl Into<String>, value: impl Into<String>) -> Field {
        Field::of(name, label, Entry::Fixed(value.into()))
    }

    pub fn text(name: &'static str, label: impl Into<String>, value: impl Into<String>) -> Field {
        Field::of(name, label, Entry::Text(value.into()))
    }

    pub fn switch(name: &'static str, label: impl Into<String>, on: bool) -> Field {
        Field::of(name, label, Entry::Switch(on))
    }

    pub fn choices(name: &'static str, label: impl Into<String>, choices: Vec<Choice>) -> Field {
        Field::of(name, label, Entry::Choices(choices))
    }

    pub fn hinted(self, hint: impl Into<String>) -> Field {
        Field {
            hint: Some(hint.into()),
            ..self
        }
    }

    pub fn changed(&self) -> bool {
        self.entry != self.was
    }

    pub fn editable(&self) -> bool {
        !matches!(self.entry, Entry::Fixed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_nobody_touched_has_not_changed_and_one_that_was_typed_into_has() {
        let mut shell = Field::text("shell", "shell", "/bin/bash");
        assert!(!shell.changed());

        shell.entry = Entry::Text("/bin/sh".into());
        assert!(
            shell.changed(),
            "what the reading said is kept beside what is typed, so a save sends only what \
             a person changed and not the whole row back"
        );
    }

    #[test]
    fn a_fixed_field_is_shown_and_not_offered_for_typing() {
        assert!(!Field::fixed("name", "name", "deploy").editable());
        assert!(Field::switch("locked", "locked", false).editable());
        assert!(Field::choices("groups", "groups", vec![Choice::of("wheel", true)]).editable());
    }
}
