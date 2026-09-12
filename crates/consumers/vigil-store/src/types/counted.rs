#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counted {
    pub held: u64,
    pub ceiling: u64,
}

impl Counted {
    pub fn new(held: u64, ceiling: u64) -> Self {
        Counted { held, ceiling }
    }

    pub fn is_empty(self) -> bool {
        self.held == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_store_with_nothing_in_it_is_empty_whatever_its_ceiling_is() {
        assert!(Counted::new(0, 10_000).is_empty());
        assert!(!Counted::new(1, 10_000).is_empty());
    }
}
