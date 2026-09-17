#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    Title { lead: String, text: String },
    Heading(String),
    Field { name: String, value: String },
    Text(String),
    Warning(String),
    Key(String),
    Line(String),
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

    pub fn line(drawn: impl Into<String>) -> Piece {
        Piece::Line(drawn.into())
    }

    pub fn text(text: impl Into<String>) -> Piece {
        Piece::Text(text.into())
    }

    pub fn warning(text: impl Into<String>) -> Piece {
        Piece::Warning(text.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drawn_line_is_a_piece_of_its_own_because_prose_is_the_only_thing_that_may_be_rewrapped() {
        let drawn = Piece::line("   in ──▶ prerouting ──▶ input");

        assert_eq!(drawn, Piece::Line("   in ──▶ prerouting ──▶ input".into()));
        assert_ne!(
            drawn,
            Piece::text("   in ──▶ prerouting ──▶ input"),
            "a diagram handed over as text is a diagram the renderer is free to wrap at the \
             width of the panel, and a wrapped diagram is a column of arrows pointing nowhere"
        );
    }
}
