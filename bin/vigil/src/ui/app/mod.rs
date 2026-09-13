#[cfg(test)]
mod tests;

mod answers;
mod choosing;
mod detail;
mod drawing;
mod jump;
mod keys;
mod levels;
mod motion;
mod narrowing;
mod page;
mod pane;
mod picking;
mod printing;
mod rows;
mod session;

use std::cell::Cell;

use ratatui::layout::Rect;

use crate::link::Link;
use std::collections::BTreeMap;

use crate::ui::{
    Asking, Chooser, Dismissed, Filter, Gone, Level, Look, Nav, Picked, Screen, Sorting, View,
};

pub struct App {
    link: Link,
    look: Look,
    nav: Nav,
    view: View,
    filter: Filter,
    picked: Picked,
    dismissed: Dismissed,
    asking: Option<Asking>,
    named_configuration: Option<String>,
    detail_open: bool,
    level: Level,
    gone: Option<Gone>,
    chooser: Chooser,
    sorting: BTreeMap<Screen, Sorting>,
    helping: bool,
    message: Option<String>,
    body: Cell<Rect>,
    refresh_wanted: bool,
    leaving: bool,
}
