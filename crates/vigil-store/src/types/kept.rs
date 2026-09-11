use super::counted::Counted;
use super::dropped::Dropped;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kept {
    pub records: Counted,
    pub bytes: Counted,
    pub oldest_at: Option<String>,
    pub dropped: Dropped,
    pub damaged: u64,
    pub compactions: u64,
}

impl Kept {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_store_with_nothing_in_it_says_so_and_does_not_answer_zero_bytes() {
        let nothing = Kept {
            records: Counted::new(0, 10_000),
            bytes: Counted::new(0, 16 * 1024 * 1024),
            ..Kept::default()
        };

        assert!(nothing.is_empty());
        assert_eq!(nothing.oldest_at, None, "and names no oldest record");
    }
}
