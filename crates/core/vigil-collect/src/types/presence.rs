#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    OnDisk,
    Gone,
    NotShown,
}

impl Presence {
    pub fn on_disk(self) -> Option<bool> {
        match self {
            Presence::OnDisk => Some(true),
            Presence::Gone => Some(false),
            Presence::NotShown => None,
        }
    }

    pub fn shown(self) -> bool {
        self != Presence::NotShown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_the_agent_was_not_shown_does_not_answer_whether_the_file_is_there() {
        assert_eq!(Presence::NotShown.on_disk(), None);
        assert!(!Presence::NotShown.shown());
    }

    #[test]
    fn not_seeing_a_file_and_seeing_that_it_is_gone_are_two_answers() {
        assert_eq!(Presence::Gone.on_disk(), Some(false));
        assert_eq!(Presence::OnDisk.on_disk(), Some(true));
        assert!(Presence::Gone.shown());
        assert_ne!(Presence::Gone, Presence::NotShown);
    }
}
