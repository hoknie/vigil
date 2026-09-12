use super::counted::Counted;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Held {
    pub records: Counted,
    pub bytes: Counted,
    pub dropped: u64,
    pub damaged: u64,
    pub oldest_at: Option<String>,
}

impl Held {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn at_the_ceiling(&self) -> bool {
        self.records.held >= self.records.ceiling || self.bytes.held >= self.bytes.ceiling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buffer_that_holds_nothing_is_not_a_buffer_that_was_never_opened() {
        let empty = Held {
            records: Counted::new(0, 2_000),
            bytes: Counted::new(0, 4 * 1024 * 1024),
            ..Held::default()
        };

        assert!(empty.is_empty());
        assert!(!empty.at_the_ceiling());
        assert_eq!(
            empty.oldest_at, None,
            "and it names no oldest finding, rather than the epoch"
        );
    }

    #[test]
    fn either_ceiling_on_its_own_means_the_next_finding_displaces_one() {
        let by_bytes = Held {
            records: Counted::new(12, 2_000),
            bytes: Counted::new(4 * 1024 * 1024, 4 * 1024 * 1024),
            ..Held::default()
        };

        assert!(
            by_bytes.at_the_ceiling(),
            "twelve enormous findings fill a buffer as surely as two thousand small ones"
        );
    }
}
