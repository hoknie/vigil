#[cfg(test)]
mod tests;

mod answers;
mod choices;
mod deeds;
mod drawing;
mod history;
mod kept;
mod keys;
mod narrowing;
mod navigation;
mod page;
mod pane;
mod rows;
mod session;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ratatui::layout::Rect;
use vigil_view::{Counts, Index, Piece, RowKey};

use crate::link::Link;
use std::collections::BTreeMap;

use crate::ui::types::cache::{Listed, Remembered, Shown};
use crate::ui::{
    Asking, Chooser, Dismissed, Editing, Filter, Gone, History, Level, Look, Nav, Paper, Picked,
    Screen, Sorting, View,
};

type RowsAsked = (u64, Screen, usize, String);

type DetailAsked = (u64, Screen, usize, RowKey, usize);

type FoundBefore = (RowsAsked, String, Rc<Vec<usize>>);

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
    history: Option<History>,
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
    rows_seen: Remembered<RowsAsked, Rc<Shown>>,
    listed_seen: Remembered<RowsAsked, Rc<Listed>>,
    index_seen: Remembered<RowsAsked, Option<Rc<Index>>>,
    found_seen: RefCell<Option<FoundBefore>>,
    tally_seen: Remembered<RowsAsked, String>,
    counts_seen: Remembered<RowsAsked, Option<Rc<Counts>>>,
    detail_seen: Remembered<DetailAsked, Rc<Vec<Piece>>>,
    refresh_wanted: bool,
    leaving: bool,
}
