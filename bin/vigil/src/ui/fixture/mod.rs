#[cfg(test)]
mod tests;

pub mod agent;
pub mod answers;
pub mod findings;
pub mod host;
pub mod look;
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
