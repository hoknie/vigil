use super::emphasis::Emphasis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowKey {
    pub key: String,
    pub emphasis: Emphasis,
    pub depth: u8,
    pub of_the_reading: bool,
    pub gathers: Option<usize>,
    pub gathered_under: Option<String>,
    pub opened: bool,
}

impl RowKey {
    pub fn of(key: impl Into<String>) -> RowKey {
        RowKey {
            key: key.into(),
            emphasis: Emphasis::Plain,
            depth: 0,
            of_the_reading: true,
            gathers: None,
            gathered_under: None,
            opened: false,
        }
    }

    pub fn gathering(self, rows: usize) -> RowKey {
        RowKey {
            gathers: Some(rows),
            ..self
        }
    }

    pub fn opened(self, opened: bool) -> RowKey {
        RowKey { opened, ..self }
    }

    pub fn beneath(self, heading: impl Into<String>) -> RowKey {
        RowKey {
            gathered_under: Some(heading.into()),
            ..self
        }
    }

    pub fn opens(&self) -> bool {
        self.gathers.is_some_and(|rows| rows > 0)
    }

    pub fn said(self, emphasis: Emphasis) -> RowKey {
        RowKey { emphasis, ..self }
    }

    pub fn under(self, depth: u8) -> RowKey {
        RowKey { depth, ..self }
    }

    pub fn of_its_own(self) -> RowKey {
        RowKey {
            of_the_reading: false,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_heading_that_gathers_nothing_does_not_offer_to_be_opened() {
        assert!(!RowKey::of("program|/usr/sbin/nginx").gathering(0).opens());
        assert!(RowKey::of("program|/usr/sbin/nginx").gathering(3).opens());
        assert!(
            !RowKey::of("tcp|0.0.0.0:80").opens(),
            "a socket is the leaf, and an arrow that opens a leaf is an arrow that lies"
        );
    }

    #[test]
    fn a_row_under_a_heading_names_the_heading_so_the_arrow_back_knows_where_to_land() {
        let row = RowKey::of("tcp|0.0.0.0:80")
            .under(1)
            .beneath("program|/usr/sbin/nginx");

        assert_eq!(
            row.gathered_under.as_deref(),
            Some("program|/usr/sbin/nginx"),
            "walking back up by counting depth backwards reads the row above, which is the \
             next socket of the same program, not the heading over both"
        );
    }
}
