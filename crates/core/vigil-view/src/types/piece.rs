#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    Title { lead: String, text: String },
    Heading(String),
    Field { name: String, value: String },
    Text(String),
    Warning(String),
    Key(String),
    Blank,
}

impl Piece {
    pub fn title(lead: impl Into<String>, text: impl Into<String>) -> Piece {
        Piece::Title {
            lead: lead.into(),
            text: text.into(),
        }
    }

    pub fn key(key: impl Into<String>) -> Piece {
        Piece::Key(key.into())
    }

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
