use super::Crossing;

const UNDER_IN_A_ROW_TO_CLOSE: u8 = 3;

#[derive(Debug, Default)]
pub struct Gate {
    over: bool,
    under_in_a_row: u8,
}

impl Gate {
    pub fn judge(&mut self, over: bool) -> Option<Crossing> {
        if over {
            self.under_in_a_row = 0;
            if self.over {
                return None;
            }
            self.over = true;
            return Some(Crossing::Exceeded);
        }

        if !self.over {
            return None;
        }
        self.under_in_a_row += 1;
        if self.under_in_a_row < UNDER_IN_A_ROW_TO_CLOSE {
            return None;
        }
        self.over = false;
        self.under_in_a_row = 0;
        Some(Crossing::Recovered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staying_over_the_ceiling_is_said_once_and_not_at_every_check() {
        let mut gate = Gate::default();

        assert_eq!(gate.judge(true), Some(Crossing::Exceeded));
        assert_eq!(gate.judge(true), None);
        assert_eq!(gate.judge(true), None);
    }

    #[test]
    fn a_reading_under_the_ceiling_does_not_close_a_finding_on_its_own() {
        let mut gate = Gate::default();
        gate.judge(true);

        assert_eq!(
            gate.judge(false),
            None,
            "one check is a wobble, not a return"
        );
        assert_eq!(gate.judge(false), None);
        assert_eq!(gate.judge(false), Some(Crossing::Recovered));
    }

    #[test]
    fn a_single_check_back_over_the_ceiling_starts_the_count_again() {
        let mut gate = Gate::default();
        gate.judge(true);
        gate.judge(false);
        gate.judge(false);

        assert_eq!(gate.judge(true), None, "it never stopped being over");
        assert_eq!(gate.judge(false), None);
        assert_eq!(gate.judge(false), None);
        assert_eq!(gate.judge(false), Some(Crossing::Recovered));
    }

    #[test]
    fn an_agent_that_was_never_over_its_ceiling_has_nothing_to_close() {
        let mut gate = Gate::default();

        for _ in 0..10 {
            assert_eq!(gate.judge(false), None);
        }
    }
}
