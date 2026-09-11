#[cfg(test)]
mod tests;

mod answers;
mod detail;
mod drawing;
mod jump;
mod keys;
mod levels;
mod motion;
mod narrowing;
mod page;
mod printing;
mod rows;
mod session;

use std::cell::Cell;

use ratatui::layout::Rect;

use crate::link::Link;
use crate::ui::{Filter, Level, Look, Nav, Nesting, Protocols, View};

pub struct App {
    link: Link,
    look: Look,
    nav: Nav,
    view: View,
    filter: Filter,
    detail_open: bool,
    level: Level,
    ports_protocols: Protocols,
    startup_nesting: Nesting,
    helping: bool,
    message: Option<String>,
    body: Cell<Rect>,
    refresh_wanted: bool,
    leaving: bool,
}
