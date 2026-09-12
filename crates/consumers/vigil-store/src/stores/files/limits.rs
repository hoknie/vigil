#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub findings: usize,
    pub journal_bytes: u64,
}

impl Limits {
    pub fn low_water(self) -> usize {
        self.findings - self.findings / 10
    }

    pub fn low_water_bytes(self) -> u64 {
        self.journal_bytes - self.journal_bytes / 10
    }

    pub fn outgoing() -> Self {
        Limits {
            findings: 500,
            journal_bytes: 4 * 1024 * 1024,
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            findings: 10_000,
            journal_bytes: 16 * 1024 * 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_one_receiver_may_hold_is_a_fraction_of_what_the_history_holds() {
        let outgoing = Limits::outgoing();
        let history = Limits::default();

        assert!(
            outgoing.findings < history.findings && outgoing.journal_bytes < history.journal_bytes,
            "the history is one file on this host; the buffer is one file per receiver, and \
             every one of them is held in memory as well as on disk"
        );
    }

    #[test]
    fn compaction_leaves_room_to_write_into_rather_than_stopping_at_the_line() {
        let limits = Limits::outgoing();

        assert!(limits.low_water() < limits.findings);
        assert!(
            limits.low_water_bytes() < limits.journal_bytes,
            "a buffer compacted exactly to its ceiling compacts again on the next finding"
        );
    }
}
