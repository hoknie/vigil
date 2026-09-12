#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    Heading(String),
    Field { name: String, value: String },
    Text(String),
    Warning(String),
    Key(String),
    Blank,
}

impl Piece {
    pub fn heading(text: impl Into<String>) -> Piece {
        Piece::Heading(text.into())
    }

    pub fn field(name: impl Into<String>, value: impl Into<String>) -> Piece {
        Piece::Field {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn text(text: impl Into<String>) -> Piece {
        Piece::Text(text.into())
    }

    pub fn warning(text: impl Into<String>) -> Piece {
        Piece::Warning(text.into())
    }
}
