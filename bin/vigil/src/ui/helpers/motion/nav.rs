use super::lists::Lists;
use crate::ui::{Cursor, Offset, Origin, Screen};

#[derive(Debug, Clone, Default)]
pub struct Nav {
    at: Screen,
    origin: Option<Origin>,
    pub sections: Cursor,
    pub lists: Lists,
    pub firewall: Cursor,
    pub findings: Cursor,
    pub summary: Offset,
    pub difference: Offset,
}

impl Nav {
    pub fn opening(at: Screen) -> Self {
        Nav {
            at,
            ..Nav::default()
        }
    }

    pub fn at(&self) -> Screen {
        self.at
    }

    pub fn visit(&mut self, screen: Screen) {
        self.at = screen;
        self.origin = None;
    }

    pub fn jump(&mut self, to: Screen, from: Origin) {
        if to == self.at {
            return;
        }
        self.at = to;
        self.origin = Some(from);
    }

    pub fn came_from(&self) -> Option<&Origin> {
        self.origin.as_ref()
    }

    pub fn came_back(&mut self) -> Option<Origin> {
        let origin = self.origin.take()?;
        self.at = origin.screen();
        Some(origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_a_finding() -> Origin {
        Origin::new(Screen::Findings, "0199a1b2-c3d4-7e5f-8a9b-000000000001")
    }

    #[test]
    fn escape_after_a_jump_comes_back_to_the_row_it_was_pressed_on() {
        let mut nav = Nav::opening(Screen::Findings);

        nav.jump(Screen::Ports, from_a_finding());
        assert_eq!(nav.at(), Screen::Ports);

        let back = nav.came_back().expect("somewhere to come back to");
        assert_eq!(nav.at(), Screen::Findings);
        assert_eq!(back.key(), "0199a1b2-c3d4-7e5f-8a9b-000000000001");
    }

    #[test]
    fn a_jump_remembers_one_place_and_forgets_it_once_it_has_been_used() {
        let mut nav = Nav::opening(Screen::Findings);

        nav.jump(Screen::Ports, from_a_finding());
        nav.came_back();

        assert!(
            nav.came_back().is_none(),
            "a second Escape from the same rung goes to the main screen, not round a ring"
        );
    }

    #[test]
    fn walking_into_a_section_from_the_main_screen_leaves_nothing_for_escape_to_come_back_to() {
        let mut nav = Nav::opening(Screen::Home);

        nav.visit(Screen::Accounts);

        assert!(nav.came_back().is_none());
        assert_eq!(nav.at(), Screen::Accounts);
    }

    #[test]
    fn changing_section_by_its_number_forgets_where_the_jump_came_from() {
        let mut nav = Nav::opening(Screen::Findings);
        nav.jump(Screen::Ports, from_a_finding());

        nav.visit(Screen::Accounts);

        assert!(nav.came_from().is_none());
        assert!(nav.came_back().is_none());
    }

    #[test]
    fn jumping_into_the_section_already_open_records_nothing_to_come_back_from() {
        let mut nav = Nav::opening(Screen::Ports);

        nav.jump(Screen::Ports, from_a_finding());

        assert!(nav.came_back().is_none());
    }
}
