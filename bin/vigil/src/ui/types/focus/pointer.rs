use std::cell::{Cell, RefCell};

use ratatui::layout::Rect;

use crate::ui::{Target, Targets};

#[derive(Debug)]
pub struct Pointer {
    on: bool,
    drawn: Cell<bool>,
    targets: RefCell<Targets>,
}

impl Pointer {
    pub fn starting(on: bool) -> Pointer {
        Pointer {
            on,
            drawn: Cell::new(false),
            targets: RefCell::new(Targets::default()),
        }
    }

    pub fn on(&self) -> bool {
        self.on
    }

    pub fn switch(&mut self) {
        self.on = !self.on;
    }

    pub fn drawing(&self) {
        self.targets.borrow_mut().clear();
    }

    pub fn put(&self, area: Rect, target: Target) {
        self.targets.borrow_mut().put(area, target);
    }

    pub fn finished(&self) {
        self.drawn.set(true);
    }

    pub fn resized(&self) {
        self.drawn.set(false);
        self.targets.borrow_mut().clear();
    }

    pub fn ready(&self) -> bool {
        self.drawn.get()
    }

    pub fn first_under(&self, column: u16, row: u16) -> Option<Target> {
        self.targets.borrow().first_under(column, row)
    }

    pub fn under(&self, column: u16, row: u16) -> Vec<Target> {
        self.targets.borrow().under(column, row).collect()
    }

    #[cfg(test)]
    pub fn recorded(&self) -> usize {
        self.targets.borrow().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_clicked_on_a_screen_that_has_not_been_drawn_since_it_changed_size() {
        let pointer = Pointer::starting(true);
        assert!(
            !pointer.ready(),
            "a console that has drawn nothing has nothing to click"
        );

        pointer.drawing();
        pointer.put(Rect::new(0, 0, 10, 1), Target::Back);
        pointer.finished();
        assert!(pointer.ready());
        assert_eq!(pointer.first_under(3, 0), Some(Target::Back));

        pointer.resized();
        assert!(
            !pointer.ready(),
            "the places recorded belong to a screen of another size, and a click read against \
             them lands on whatever used to be there"
        );
        assert_eq!(pointer.first_under(3, 0), None);
    }
}
