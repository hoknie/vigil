#[cfg(test)]
mod tests;

pub mod agent;
pub mod answers;
pub mod findings;
pub mod grouped;
pub mod host;
pub mod look;
pub mod readings;
pub mod store;
pub mod view;

use super::{Audience, Look, Palette, Reading, Screen, Status, View};

pub use agent::{behind, collector_off, losing};
pub use findings::finding;
pub use look::{look, monochrome};
pub use store::store;
pub use view::{view, view_with_launches, view_with_trouble};

pub fn screen(named: &str) -> Screen {
    Screen::parse(named)
        .unwrap_or_else(|| panic!("{named} is not a screen this build of the console draws"))
}

pub fn of_a_reading_with_no_screen() -> (Status, Reading) {
    let mut status = Status {
        host: host::host(),
        agent: agent::agent(),
        sent_at: "2026-09-09T09:00:01.000Z".into(),
    };
    status
        .agent
        .collectors
        .retain(|collector| crate::ui::Screen::showing(&collector.name).is_some());
    status
        .agent
        .collectors
        .push(agent::collector("kernel", 60, 2, 0));

    (status, Reading::Taken(readings::with_no_screen()))
}
