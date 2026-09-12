use super::emphasis::Emphasis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowKey {
    pub key: String,
    pub emphasis: Emphasis,
    pub depth: u8,
    pub selectable: bool,
}

impl RowKey {
    pub fn of(key: impl Into<String>) -> RowKey {
        RowKey {
            key: key.into(),
            emphasis: Emphasis::Plain,
            depth: 0,
            selectable: true,
        }
    }

    pub fn said(self, emphasis: Emphasis) -> RowKey {
        RowKey { emphasis, ..self }
    }

    pub fn under(self, depth: u8) -> RowKey {
        RowKey { depth, ..self }
    }

    pub fn heading(self) -> RowKey {
        RowKey {
            selectable: false,
            ..self
        }
    }
}
