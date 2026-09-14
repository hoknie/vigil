use crate::ui::chrome::frame::Hints;
use crate::ui::chrome::frame::hints::Back;
use crate::ui::{Level, Screen};

pub(super) fn a_section() -> Screen {
    Screen::parse("ports").expect("a section every build of this console has")
}

pub(super) fn hints(level: Level, back: Back) -> Hints<'static> {
    Hints {
        level,
        back,
        ..Hints::default()
    }
}

pub(super) fn sorting(level: Level, back: Back) -> Hints<'static> {
    Hints {
        sorts: true,
        filters: true,
        ..hints(level, back)
    }
}

pub(super) fn acting(level: Level, back: Back) -> Hints<'static> {
    Hints {
        marks: true,
        kills: true,
        ..sorting(level, back)
    }
}
