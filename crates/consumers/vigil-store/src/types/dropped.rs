#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Dropped {
    pub at_the_ceiling: u64,
    pub past_the_window: u64,
}

impl Dropped {
    pub fn any(self) -> bool {
        self.at_the_ceiling > 0 || self.past_the_window > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_ceilings_are_counted_apart_because_they_are_two_decisions() {
        let only_age = Dropped {
            at_the_ceiling: 0,
            past_the_window: 41,
        };

        assert!(only_age.any());
        assert_eq!(only_age.at_the_ceiling, 0);
        assert!(!Dropped::default().any());
    }
}
