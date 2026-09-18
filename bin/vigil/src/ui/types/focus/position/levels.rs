#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Level {
    Groups,
    Menu,
    #[default]
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rungs {
    pub groups: bool,
    pub menu: bool,
    pub detail: bool,
}

impl Rungs {
    pub fn new(groups: bool, menu: bool, detail: bool) -> Self {
        Rungs {
            groups,
            menu,
            detail,
        }
    }
}

impl Level {
    pub fn top(rungs: Rungs) -> Level {
        match (rungs.groups, rungs.menu) {
            (true, _) => Level::Groups,
            (false, true) => Level::Menu,
            (false, false) => Level::List,
        }
    }

    pub fn of_the_lists(rungs: Rungs) -> Level {
        match rungs.menu {
            true => Level::Menu,
            false => Level::top(rungs),
        }
    }

    pub fn deeper(self, rungs: Rungs) -> Level {
        match self {
            Level::Groups => match rungs.menu {
                true => Level::Menu,
                false => Level::List,
            },
            Level::Menu => Level::List,
            Level::List if rungs.detail => Level::Detail,
            other => other,
        }
    }

    pub fn shallower(self, rungs: Rungs) -> Option<Level> {
        match self {
            Level::Detail => Some(Level::List),
            Level::List => match (rungs.menu, rungs.groups) {
                (true, _) => Some(Level::Menu),
                (false, true) => Some(Level::Groups),
                (false, false) => None,
            },
            Level::Menu => match rungs.groups {
                true => Some(Level::Groups),
                false => None,
            },
            Level::Groups => None,
        }
    }

    pub fn is_a_row_of_names(self) -> bool {
        matches!(self, Level::Groups | Level::Menu)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrows {
    Groups,
    Menu,
    List,
    Away,
}

impl Arrows {
    pub fn at(level: Level) -> Arrows {
        match level {
            Level::Groups => Arrows::Groups,
            Level::Menu => Arrows::Menu,
            Level::List => Arrows::List,
            _ => Arrows::Away,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: Rungs = Rungs {
        groups: false,
        menu: false,
        detail: false,
    };
    const LISTED: Rungs = Rungs {
        groups: false,
        menu: false,
        detail: true,
    };
    const SUBMENUED: Rungs = Rungs {
        groups: false,
        menu: true,
        detail: true,
    };
    const GROUPED: Rungs = Rungs {
        groups: true,
        menu: true,
        detail: true,
    };
    const GROUPED_WITH_ONE_LIST: Rungs = Rungs {
        groups: true,
        menu: false,
        detail: true,
    };

    #[test]
    fn a_section_is_entered_on_its_own_top_rung_and_the_main_screen_is_a_list() {
        assert_eq!(Level::top(PLAIN), Level::List);
        assert_eq!(Level::top(LISTED), Level::List);
        assert_eq!(Level::top(SUBMENUED), Level::Menu);
        assert_eq!(Level::top(GROUPED), Level::Groups);
        assert_eq!(Level::default(), Level::List);
    }

    #[test]
    fn every_level_in_has_a_level_back_out() {
        for rungs in [PLAIN, LISTED, SUBMENUED, GROUPED, GROUPED_WITH_ONE_LIST] {
            let mut levels = vec![Level::top(rungs)];
            if rungs.groups && rungs.menu {
                levels.push(Level::Menu);
            }
            if rungs.menu || rungs.groups {
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
        assert!(Level::Groups.is_a_row_of_names());
        assert!(!Level::List.is_a_row_of_names());
        assert!(!Level::Detail.is_a_row_of_names());
    }

    #[test]
    fn a_section_with_two_rows_is_entered_on_the_groups_and_walks_down_through_the_lists() {
        assert_eq!(Level::top(GROUPED), Level::Groups);
        assert_eq!(Level::Groups.deeper(GROUPED), Level::Menu);
        assert_eq!(Level::Menu.deeper(GROUPED), Level::List);
        assert_eq!(Level::List.deeper(GROUPED), Level::Detail);

        assert_eq!(Level::Detail.shallower(GROUPED), Some(Level::List));
        assert_eq!(Level::List.shallower(GROUPED), Some(Level::Menu));
        assert_eq!(Level::Menu.shallower(GROUPED), Some(Level::Groups));
        assert_eq!(
            Level::Groups.shallower(GROUPED),
            None,
            "the row of groups is the top rung of the section, and above it is the way out"
        );
    }

    #[test]
    fn a_group_with_one_list_or_none_to_walk_leaves_no_rung_that_moves_nothing() {
        assert_eq!(
            Level::Groups.deeper(GROUPED_WITH_ONE_LIST),
            Level::List,
            "a second row with one name on it is not walked sideways, so the arrows go \
             straight into the list"
        );
        assert_eq!(
            Level::List.shallower(GROUPED_WITH_ONE_LIST),
            Some(Level::Groups)
        );
    }

    #[test]
    fn the_arrows_are_drawn_on_the_row_the_reader_is_standing_on_and_on_no_other() {
        assert_eq!(Arrows::at(Level::Groups), Arrows::Groups);
        assert_eq!(Arrows::at(Level::Menu), Arrows::Menu);
        assert_eq!(Arrows::at(Level::List), Arrows::List);
        assert_eq!(Arrows::at(Level::Detail), Arrows::Away);
    }

    #[test]
    fn a_name_pressed_on_the_second_row_leaves_the_arrows_on_that_row_and_not_on_the_groups() {
        assert_eq!(Level::of_the_lists(GROUPED), Level::Menu);
        assert_eq!(Level::of_the_lists(SUBMENUED), Level::Menu);
        assert_eq!(
            Level::of_the_lists(GROUPED_WITH_ONE_LIST),
            Level::Groups,
            "with one list in the group there is no second row to stand on"
        );
        assert_eq!(Level::of_the_lists(PLAIN), Level::List);
    }
}
