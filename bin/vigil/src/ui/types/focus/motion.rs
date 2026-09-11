#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Up,
    Down,
    PageUp,
    PageDown,
    First,
    Last,
}

impl Motion {
    pub fn distance(self, page: usize) -> Option<isize> {
        let page = page.saturating_sub(1).max(1) as isize;
        match self {
            Motion::Up => Some(-1),
            Motion::Down => Some(1),
            Motion::PageUp => Some(-page),
            Motion::PageDown => Some(page),
            Motion::First | Motion::Last => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_turn_keeps_one_line_of_overlap() {
        assert_eq!(Motion::PageDown.distance(20), Some(19));
        assert_eq!(Motion::PageUp.distance(20), Some(-19));
    }

    #[test]
    fn a_window_with_no_room_in_it_still_moves_by_one_rather_than_by_nothing() {
        assert_eq!(Motion::PageDown.distance(0), Some(1));
        assert_eq!(Motion::PageDown.distance(1), Some(1));
    }

    #[test]
    fn the_ends_of_a_list_are_positions_and_not_distances() {
        assert_eq!(Motion::First.distance(20), None);
        assert_eq!(Motion::Last.distance(20), None);
    }
}
