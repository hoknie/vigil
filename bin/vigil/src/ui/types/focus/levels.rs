#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Level {
    Menu,
    #[default]
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rungs {
    pub menu: bool,
    pub detail: bool,
}

impl Rungs {
    pub fn new(menu: bool, detail: bool) -> Self {
        Rungs { menu, detail }
    }
}

impl Level {
    pub fn top(rungs: Rungs) -> Level {
        match rungs.menu {
            true => Level::Menu,
            false => Level::List,
        }
    }

    pub fn deeper(self, rungs: Rungs) -> Level {
        match self {
            Level::Menu => Level::List,
            Level::List if rungs.detail => Level::Detail,
            other => other,
        }
    }

    pub fn shallower(self, rungs: Rungs) -> Option<Level> {
        match self {
            Level::Detail => Some(Level::List),
            Level::List => match rungs.menu {
                true => Some(Level::Menu),
                false => None,
            },
            Level::Menu => None,
        }
    }

    pub fn is_a_row_of_names(self) -> bool {
        matches!(self, Level::Menu)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrows {
    Menu,
    List,
    Away,
}

impl Arrows {
    pub fn at(level: Level) -> Arrows {
        match level {
            Level::Menu => Arrows::Menu,
            Level::List => Arrows::List,
            _ => Arrows::Away,
        }
    }
}

pub trait Choice: Copy + Default + PartialEq {
    const COUNT: usize;

    fn index(self) -> usize;

    fn step(self, by: isize, shown: &[Self]) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: Rungs = Rungs {
        menu: false,
        detail: false,
    };
    const LISTED: Rungs = Rungs {
        menu: false,
        detail: true,
    };
    const SUBMENUED: Rungs = Rungs {
        menu: true,
        detail: true,
    };

    #[test]
    fn a_section_is_entered_on_its_own_top_rung_and_the_main_screen_is_a_list() {
        assert_eq!(Level::top(PLAIN), Level::List);
        assert_eq!(Level::top(LISTED), Level::List);
        assert_eq!(Level::top(SUBMENUED), Level::Menu);
        assert_eq!(Level::default(), Level::List);
    }

    #[test]
    fn every_level_in_has_a_level_back_out() {
        for rungs in [PLAIN, LISTED, SUBMENUED] {
            let mut levels = vec![Level::top(rungs)];
            if rungs.menu {
                levels.push(Level::List);
            }
            if rungs.detail {
                levels.push(Level::Detail);
            }

            for level in levels {
                let deeper = level.deeper(rungs);
                if deeper != level {
                    assert_eq!(
                        deeper.shallower(rungs),
                        Some(level),
                        "{level:?} could not be undone with {rungs:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_screen_with_no_detail_stops_at_its_content() {
        assert_eq!(Level::List.deeper(PLAIN), Level::List);
        assert_eq!(Level::List.deeper(LISTED), Level::Detail);
    }

    #[test]
    fn a_screen_with_a_submenu_puts_it_above_the_list_and_nothing_above_that() {
        assert_eq!(Level::Menu.deeper(SUBMENUED), Level::List);
        assert_eq!(Level::List.shallower(SUBMENUED), Some(Level::Menu));
        assert_eq!(Level::Menu.shallower(SUBMENUED), None);
    }

    #[test]
    fn the_top_rung_of_a_section_leads_out_of_the_section_and_not_around_it() {
        assert_eq!(
            Level::List.shallower(LISTED),
            None,
            "a section without a row of lists leaves from its list"
        );
        assert_eq!(Level::List.shallower(PLAIN), None);
        assert_eq!(
            Level::Menu.shallower(SUBMENUED),
            None,
            "and one with a row of lists leaves from that row"
        );
    }

    #[test]
    fn the_one_row_of_names_on_a_page_is_the_one_the_arrows_walk_sideways_along() {
        assert!(Level::Menu.is_a_row_of_names());
        assert!(!Level::List.is_a_row_of_names());
        assert!(!Level::Detail.is_a_row_of_names());
    }
}
