use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    Added {
        key: String,
        after: Value,
    },
    Removed {
        key: String,
        before: Value,
    },
    Changed {
        key: String,
        before: Value,
        after: Value,
    },
}

impl Change {
    pub fn key(&self) -> &str {
        match self {
            Change::Added { key, .. }
            | Change::Removed { key, .. }
            | Change::Changed { key, .. } => key,
        }
    }
}
