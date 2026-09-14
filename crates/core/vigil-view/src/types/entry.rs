use super::choice::Choice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Fixed(String),
    Text(String),
    Switch(bool),
    Choices(Vec<Choice>),
}

impl Entry {
    pub fn chosen(&self) -> Vec<&str> {
        match self {
            Entry::Choices(choices) => choices
                .iter()
                .filter(|choice| choice.chosen)
                .map(|choice| choice.name.as_str())
                .collect(),
            _ => Vec::new(),
        }
    }
}
