#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paper {
    pub caption: String,
    pub lines: Vec<String>,
    pub copyable: bool,
}

impl Paper {
    pub fn of(caption: impl Into<String>, lines: Vec<String>) -> Paper {
        Paper {
            caption: caption.into(),
            lines,
            copyable: false,
        }
    }

    pub fn for_copying(self) -> Paper {
        Paper {
            copyable: true,
            ..self
        }
    }

    pub fn footing(&self) -> &'static str {
        match self.copyable {
            true => " select it with the mouse; any key closes this ",
            false => " any key closes this ",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheet_meant_to_be_copied_says_how_and_one_meant_to_be_read_does_not() {
        let report = Paper::of("WHAT THE AGENT DID", vec!["one line".into()]);
        let block = Paper::of("SUPPRESSIONS", vec!["suppressions:".into()]).for_copying();

        assert!(!report.footing().contains("mouse"));
        assert!(
            block.footing().contains("mouse"),
            "the console has no clipboard of its own, and a block of yaml nobody is told how \
             to take off the screen is a block nobody takes off the screen"
        );
    }
}
