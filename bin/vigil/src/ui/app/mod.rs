#[cfg(test)]
mod tests;

mod answers;
mod choices;
mod deeds;
mod drawing;
mod keys;
mod narrowing;
mod navigation;
mod page;
mod pane;
mod rows;
mod session;

use std::cell::Cell;

use ratatui::layout::Rect;

use crate::link::Link;
use std::collections::BTreeMap;

use crate::ui::{
    Asking, Chooser, Dismissed, Editing, Filter, Gone, Level, Look, Nav, Paper, Picked, Screen,
    Sorting, View,
};

pub use deeds::KILL;

pub struct App {
    link: Link,
    look: Look,
    nav: Nav,
    view: View,
    filter: Filter,
    picked: Picked,
    dismissed: Dismissed,
    asking: Option<Asking>,
    editing: Option<Editing>,
    named_configuration: Option<String>,
    detail_open: bool,
    level: Level,
    gone: Option<Gone>,
    chooser: Chooser,
    sorting: BTreeMap<Screen, Sorting>,
    paper: Option<Paper>,
    button: usize,
    in_the_buttons: bool,
    helping: bool,
    message: Option<String>,
    body: Cell<Rect>,
    refresh_wanted: bool,
    leaving: bool,
}
