use std::cell::RefCell;

use vigil_view::Section;

type Declares = fn() -> Box<dyn Section>;

thread_local! {
    static DRAWN: RefCell<Option<(&'static str, Declares)>> = const { RefCell::new(None) };
}

pub(crate) struct Standing;

pub(crate) fn drawn(named: &'static str, declares: Declares) -> Standing {
    DRAWN.with(|held| *held.borrow_mut() = Some((named, declares)));

    Standing
}

pub(crate) fn of(named: &str) -> Option<Box<dyn Section>> {
    DRAWN.with(|held| match *held.borrow() {
        Some((instead, declares)) if instead == named => Some(declares()),
        _ => None,
    })
}

impl Drop for Standing {
    fn drop(&mut self) {
        DRAWN.with(|held| *held.borrow_mut() = None);
    }
}
