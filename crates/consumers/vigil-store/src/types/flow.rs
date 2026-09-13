#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Steady,
    Dropping { dropped: u64, held: u64 },
    Drained { dropped: u64 },
}

impl Flow {
    pub fn is_steady(self) -> bool {
        matches!(self, Flow::Steady)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buffer_that_is_merely_full_is_not_a_buffer_that_is_losing_findings() {
        assert!(Flow::Steady.is_steady());
        assert!(
            !Flow::Dropping {
                dropped: 1,
                held: 2_000
            }
            .is_steady()
        );
        assert!(
            !Flow::Drained { dropped: 41 }.is_steady(),
            "the moment a buffer empties is an event, and the run after it is the steady one"
        );
    }
}
