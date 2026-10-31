use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ratatui::layout::{Position, Rect};
use vigil_view::{Counts, Index, Piece, RowKey};

use crate::link::Link;
use std::collections::BTreeMap;

use crate::ui::types::cache::{Listed, Remembered, Shown};
use crate::ui::{
    Asking, Chooser, Dismissed, Editing, Filter, Gone, Graph, History, Level, Look, Nav, Paper,
    Picked, Pointer, Screen, Silences, Sorting, View,
};

pub(super) type RowsAsked = (u64, Screen, usize, String);

pub(super) type DetailAsked = (u64, Screen, usize, RowKey, usize);

pub(super) type FoundBefore = (RowsAsked, String, Rc<Vec<usize>>);

pub struct App {
    pub(super) link: Link,
    pub(super) look: Look,
    pub(super) nav: Nav,
    pub(super) view: View,
    pub(super) filter: Filter,
    pub(super) picked: Picked,
    pub(super) dismissed: Dismissed,
    pub(super) asking: Option<Asking>,
    pub(super) editing: Option<Editing>,
    pub(super) graph: Option<Graph>,
    pub(super) history: Option<History>,
    pub(super) silences: Option<Silences>,
    pub(super) named_configuration: Option<String>,
    pub(super) detail_open: bool,
    pub(super) level: Level,
    pub(super) gone: Option<Gone>,
    pub(super) chooser: Chooser,
    pub(super) sorting: BTreeMap<Screen, Sorting>,
    pub(super) paper: Option<Paper>,
    pub(super) button: usize,
    pub(super) in_the_buttons: bool,
    pub(super) helping: bool,
    pub(super) message: Option<String>,
    pub(super) body: Cell<Rect>,
    pub(super) cursor: Cell<Option<Position>>,
    pub(super) pointer: Pointer,
    pub(super) rows_seen: Remembered<RowsAsked, Rc<Shown>>,
    pub(super) listed_seen: Remembered<RowsAsked, Rc<Listed>>,
    pub(super) index_seen: Remembered<RowsAsked, Option<Rc<Index>>>,
    pub(super) found_seen: RefCell<Option<FoundBefore>>,
    pub(super) tally_seen: Remembered<RowsAsked, String>,
    pub(super) counts_seen: Remembered<RowsAsked, Option<Rc<Counts>>>,
    pub(super) detail_seen: Remembered<DetailAsked, Rc<Vec<Piece>>>,
    pub(super) refresh_wanted: bool,
    pub(super) leaving: bool,
}
