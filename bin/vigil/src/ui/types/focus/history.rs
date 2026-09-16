use vigil_view::Piece;

use crate::ui::Motion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    pieces: Vec<Piece>,
    top: usize,
}

impl History {
    pub fn open(pieces: Vec<Piece>) -> History {
        History { pieces, top: 0 }
    }

    pub fn pieces(&self) -> &[Piece] {
        &self.pieces
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn scroll(&mut self, motion: Motion, page: usize, total: usize) {
        let last = total.saturating_sub(page);
        self.top = match (motion.distance(page), motion) {
            (Some(by), _) => self.top.saturating_add_signed(by).min(last),
            (None, Motion::First) => 0,
            (None, _) => last,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opened() -> History {
        History::open(vec![Piece::Blank; 30])
    }

    #[test]
    fn scrolling_stops_at_the_last_page_so_the_first_key_back_moves_the_panel_at_once() {
        let mut history = opened();
        for _ in 0..50 {
            history.scroll(Motion::Down, 10, 30);
        }
        assert_eq!(history.top(), 20);

        history.scroll(Motion::Up, 10, 30);
        assert_eq!(
            history.top(),
            19,
            "a panel that counted presses past its end makes the reader press as many times \
             again before anything moves"
        );
    }

    #[test]
    fn the_two_ends_of_a_history_are_one_press_away() {
        let mut history = opened();

        history.scroll(Motion::Last, 10, 30);
        assert_eq!(history.top(), 20);
        history.scroll(Motion::First, 10, 30);
        assert_eq!(history.top(), 0);
    }

    #[test]
    fn a_history_shorter_than_the_panel_does_not_scroll_at_all() {
        let mut history = opened();

        history.scroll(Motion::PageDown, 40, 30);

        assert_eq!(
            history.top(),
            0,
            "a panel scrolled past a history that fits it shows an empty page"
        );
    }
}
