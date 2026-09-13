use vigil_view::{Pane, RowKey, Showing};

use crate::ui::{Reading, View};

pub fn rows(view: &View, pane: &dyn Pane, showing: &Showing<'_>) -> Vec<RowKey> {
    let Reading::Taken(snapshot) = view.reading(pane.reads()) else {
        return Vec::new();
    };
    pane.rows(snapshot, showing)
}
