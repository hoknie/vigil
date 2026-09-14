#[cfg(test)]
mod tests;

mod answers;
mod choices;
mod deeds;
mod drawing;
mod kept;
mod keys;
mod narrowing;
mod navigation;
mod page;
mod pane;
mod rows;
mod session;

use std::cell::Cell;
use std::rc::Rc;

use ratatui::layout::Rect;
use vigil_view::{Piece, RowKey};

use crate::link::Link;
use std::collections::BTreeMap;

use crate::ui::types::cache::Remembered;
use crate::ui::{
    Asking, Chooser, Dismissed, Editing, Filter, Gone, Level, Look, Nav, Paper, Picked, Screen,
    Sorting, View,
};

type RowsAsked = (u64, Screen, usize, String);

type DetailAsked = (u64, Screen, usize, RowKey, usize);

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
    rows_seen: Remembered<RowsAsked, Rc<Vec<RowKey>>>,
    tally_seen: Remembered<RowsAsked, String>,
    detail_seen: Remembered<DetailAsked, Rc<Vec<Piece>>>,
    refresh_wanted: bool,
    leaving: bool,
}
