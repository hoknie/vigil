use ratatui::layout::{Position, Rect};

use crate::ui::Target;

#[derive(Debug, Default)]
pub struct Targets {
    drawn: Vec<(Rect, Target)>,
}

impl Targets {
    pub fn clear(&mut self) {
        self.drawn.clear();
    }

    pub fn put(&mut self, area: Rect, target: Target) {
        if area.width > 0 && area.height > 0 {
            self.drawn.push((area, target));
        }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.drawn.is_empty()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.drawn.len()
    }

    pub fn under(&self, column: u16, row: u16) -> impl Iterator<Item = Target> + '_ {
        let position = Position { x: column, y: row };
        self.drawn
            .iter()
            .rev()
            .filter(move |(area, _)| area.contains(position))
            .map(|(_, target)| *target)
    }

    pub fn first_under(&self, column: u16, row: u16) -> Option<Target> {
        self.under(column, row).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Aim;

    #[test]
    fn what_was_drawn_last_is_what_a_click_lands_on() {
        let mut targets = Targets::default();
        targets.put(Rect::new(0, 0, 80, 24), Target::List);
        targets.put(Rect::new(2, 3, 20, 1), Target::Row(1));
        targets.put(Rect::new(0, 2, 30, 6), Target::Aim(Aim::Popup));
        targets.put(Rect::new(1, 3, 28, 1), Target::Aim(Aim::Option(0)));

        assert_eq!(
            targets.first_under(4, 3),
            Some(Target::Aim(Aim::Option(0))),
            "a popup drawn over a row is what the reader sees there, so it is what is clicked"
        );
        assert_eq!(
            targets.under(4, 3).collect::<Vec<_>>(),
            vec![
                Target::Aim(Aim::Option(0)),
                Target::Aim(Aim::Popup),
                Target::Row(1),
                Target::List
            ]
        );
        assert_eq!(targets.first_under(50, 3), Some(Target::List));
        assert_eq!(targets.first_under(90, 3), None);
    }

    #[test]
    fn a_place_with_no_room_is_not_kept_and_a_cleared_frame_holds_nothing() {
        let mut targets = Targets::default();
        targets.put(Rect::new(0, 0, 0, 1), Target::Back);
        assert!(targets.is_empty(), "nothing can be clicked in no columns");

        targets.put(Rect::new(0, 0, 10, 1), Target::Back);
        assert_eq!(targets.len(), 1);
        targets.clear();
        assert_eq!(
            targets.first_under(0, 0),
            None,
            "a new frame starts with nothing to click until it draws it"
        );
    }
}
