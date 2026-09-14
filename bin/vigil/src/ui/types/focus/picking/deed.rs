#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deed {
    Remove,
}

impl Deed {
    pub const ALL: &'static [Deed] = &[Deed::Remove];

    pub fn of(key: char) -> Option<Deed> {
        Deed::ALL.iter().copied().find(|deed| deed.key() == key)
    }

    pub fn key(self) -> char {
        match self {
            Deed::Remove => 'd',
        }
    }

    pub fn named(self) -> &'static str {
        match self {
            Deed::Remove => "silence them and take them off",
        }
    }

    pub fn alone(self) -> &'static str {
        match self {
            Deed::Remove => "silence it in the configuration, and take it off this screen",
        }
    }

    pub fn briefly(self) -> &'static str {
        match self {
            Deed::Remove => "silence",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_deed_is_reached_by_the_key_the_bar_and_the_panel_draw_beside_it() {
        for deed in Deed::ALL {
            assert_eq!(Deed::of(deed.key()), Some(*deed));
        }
        assert_eq!(Deed::of('z'), None);
    }

    #[test]
    fn no_two_deeds_answer_to_the_same_key() {
        let mut keys: Vec<char> = Deed::ALL.iter().map(|deed| deed.key()).collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();

        assert_eq!(keys.len(), before);
    }

    #[test]
    fn a_deed_is_named_for_one_row_and_for_several_because_it_reaches_both() {
        for deed in Deed::ALL {
            assert_ne!(deed.alone(), deed.named());
            assert!(!deed.alone().is_empty() && !deed.briefly().is_empty());
        }
    }

    #[test]
    fn the_one_deed_this_console_can_do_says_that_it_reaches_further_than_the_screen() {
        for said in [Deed::Remove.named(), Deed::Remove.alone()] {
            assert!(
                said.contains("silence"),
                "a row that comes back at the next reading is not removed, and the word must \
                 not promise what only the configuration can keep: {said}"
            );
        }
    }
}
