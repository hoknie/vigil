use super::entry::Entry;
use super::field::Field;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    pub caption: String,
    pub about: Vec<String>,
    pub fields: Vec<Field>,
}

impl Form {
    pub fn new(caption: impl Into<String>) -> Form {
        Form {
            caption: caption.into(),
            about: Vec::new(),
            fields: Vec::new(),
        }
    }

    pub fn saying(mut self, line: impl Into<String>) -> Form {
        self.about.push(line.into());
        self
    }

    pub fn with(mut self, field: Field) -> Form {
        self.fields.push(field);
        self
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.name == name)
    }

    pub fn field_mut(&mut self, name: &str) -> Option<&mut Field> {
        self.fields.iter_mut().find(|field| field.name == name)
    }

    pub fn text(&self, name: &str) -> Option<&str> {
        match &self.field(name)?.entry {
            Entry::Text(text) | Entry::Fixed(text) => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn switch(&self, name: &str) -> Option<bool> {
        match self.field(name)?.entry {
            Entry::Switch(on) => Some(on),
            _ => None,
        }
    }

    pub fn chosen(&self, name: &str) -> Option<Vec<&str>> {
        match &self.field(name)?.entry {
            entry @ Entry::Choices(_) => Some(entry.chosen()),
            _ => None,
        }
    }

    pub fn changed(&self, name: &str) -> bool {
        self.field(name).is_some_and(Field::changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::editing::choice::Choice;

    fn form() -> Form {
        Form::new("EDIT THE ACCOUNT deploy")
            .saying("the agent runs usermod")
            .with(Field::fixed("name", "name", "deploy"))
            .with(Field::text("shell", "shell", "/bin/bash"))
            .with(Field::switch("locked", "locked", false))
            .with(Field::choices(
                "groups",
                "groups",
                vec![Choice::of("wheel", true), Choice::of("docker", false)],
            ))
    }

    #[test]
    fn each_field_is_read_back_by_its_name_in_the_shape_it_was_made_in() {
        let form = form();

        assert_eq!(form.text("name"), Some("deploy"));
        assert_eq!(form.text("shell"), Some("/bin/bash"));
        assert_eq!(form.switch("locked"), Some(false));
        assert_eq!(form.chosen("groups"), Some(vec!["wheel"]));
        assert_eq!(form.switch("shell"), None, "a text is not a switch");
        assert_eq!(form.text("nothing"), None);
    }

    #[test]
    fn a_form_says_which_of_its_fields_a_person_changed() {
        let mut form = form();
        if let Some(field) = form.field_mut("locked") {
            field.entry = Entry::Switch(true);
        }

        assert!(form.changed("locked"));
        assert!(!form.changed("shell"));
    }
}
