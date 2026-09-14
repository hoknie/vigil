use crate::ui::Motion;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Offset {
    top: usize,
}

impl Offset {
    pub fn top(&self) -> usize {
        self.top
    }

    pub fn step(&mut self, motion: Motion, total: usize, page: usize) {
        let last = total.saturating_sub(page);
        self.top = match motion.distance(page) {
            Some(distance) => self.top.saturating_add_signed(distance).min(last),
            None => match motion {
                Motion::First => 0,
                _ => last,
            },
        };
    }

    pub fn settle(&mut self, total: usize, page: usize) {
        self.top = self.top.min(total.saturating_sub(page));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_that_fits_does_not_scroll_at_all() {
        let mut offset = Offset::default();
        offset.step(Motion::Down, 10, 20);
        assert_eq!(offset.top(), 0);
        offset.step(Motion::Last, 10, 20);
        assert_eq!(offset.top(), 0);
    }

    #[test]
    fn the_last_page_is_the_end_and_not_a_screen_of_nothing() {
        let mut offset = Offset::default();
        offset.step(Motion::Last, 100, 20);
        assert_eq!(offset.top(), 80);

        for _ in 0..50 {
            offset.step(Motion::Down, 100, 20);
        }
        assert_eq!(offset.top(), 80, "there is no scrolling past the end");
    }

    #[test]
    fn a_report_that_shrank_under_the_reader_does_not_leave_them_below_it() {
        let mut offset = Offset::default();
        offset.step(Motion::Last, 100, 20);

        offset.settle(30, 20);

        assert_eq!(offset.top(), 10);
    }
}
