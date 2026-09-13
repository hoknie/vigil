use crate::helpers::quoting::quoted;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub prefix: bool,
    pub kind: Option<String>,
    pub until: Option<String>,
    pub reason: String,
}

impl Entry {
    pub fn named(&self) -> &'static str {
        match self.prefix {
            true => "finding_key_prefix",
            false => "finding_key",
        }
    }

    pub(crate) fn lines(&self) -> Vec<String> {
        let mut lines = vec![format!("  - {}: {}", self.named(), quoted(&self.key))];
        if let Some(kind) = &self.kind {
            lines.push(format!("    kind: {}", quoted(kind)));
        }
        if let Some(until) = &self.until {
            lines.push(format!("    until: {}", quoted(until)));
        }
        lines.push(format!("    reason: {}", quoted(&self.reason)));
        lines
    }
}
