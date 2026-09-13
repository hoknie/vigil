use super::emphasis::Emphasis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub text: String,
    pub emphasis: Emphasis,
}

impl Cell {
    pub fn plain(text: impl Into<String>) -> Cell {
        Cell {
            text: text.into(),
            emphasis: Emphasis::Plain,
        }
    }

    pub fn said(text: impl Into<String>, emphasis: Emphasis) -> Cell {
        Cell {
            text: text.into(),
            emphasis,
        }
    }
}
