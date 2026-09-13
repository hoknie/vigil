use super::text::{clamp, escape, printable};

const FIELD_VALUE_CAP: usize = 512;

#[derive(Default)]
pub(super) struct Fields {
    pub(super) items: Vec<String>,
    pub(super) clamped: usize,
}

impl Fields {
    pub(super) fn pair(&mut self, name: &str, value: &str) {
        let name: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .collect();
        let (value, cut) = clamp(&escape(&printable(value)), FIELD_VALUE_CAP);
        self.clamped += cut;
        self.items.push(format!(
            "{}=\"{value}\"",
            match name.is_empty() {
                true => "field".to_string(),
                false => name,
            }
        ));
    }
}
