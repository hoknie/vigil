#[cfg(test)]
mod tests;

mod answers;
mod choices;
pub(in crate::ui) mod clicks;
mod deeds;
mod drawing;
mod graph;
mod groups;
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

use ratatui::layout::{Position, Rect};
use vigil_view::{Counts, Index, Piece, RowKey};

use crate::link::Link;
use std::collections::BTreeMap;

use crate::ui::types::cache::{Listed, Remembered, Shown};
use crate::ui::{
    Asking, Chooser, Dismissed, Editing, Filter, Gone, Graph, History, Level, Look, Nav, Paper,
    Picked, Pointer, Screen, Sorting, View,
};

type RowsAsked = (u64, Screen, usize, String);

type DetailAsked = (u64, Screen, usize, RowKey, usize);

type FoundBefore = (RowsAsked, String, Rc<Vec<usize>>);

pub use deeds::KILL;
pub use graph::WATCHING;

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
    graph: Option<Graph>,
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
    cursor: Cell<Option<Position>>,
    pointer: Pointer,
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
