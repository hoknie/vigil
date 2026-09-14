use vigil_view::{Entry, Form};

pub const NOTHING_CHANGED: &str = "nothing was changed: every field still says what the reading says, so there is nothing to \
     ask the agent for";

pub fn typed(form: &Form, name: &str) -> String {
    form.text(name).unwrap_or_default().trim().to_string()
}

pub fn changed_text(form: &Form, name: &str) -> Option<String> {
    match form.changed(name) {
        true => Some(typed(form, name)),
        false => None,
    }
}

pub fn chosen(form: &Form, name: &str) -> Vec<String> {
    form.chosen(name)
        .unwrap_or_default()
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub fn unchosen(form: &Form, name: &str) -> Vec<String> {
    match form.field(name).map(|field| &field.entry) {
        Some(Entry::Choices(choices)) => choices
            .iter()
            .filter(|choice| !choice.chosen)
            .map(|choice| choice.name.clone())
            .collect(),
        _ => Vec::new(),
    }
}
