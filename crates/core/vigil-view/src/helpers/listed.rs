use vigil_model::Snapshot;

use crate::{Index, Pane, RowKey, Showing};

pub fn listed(
    pane: &dyn Pane,
    reading: &Snapshot,
    showing: &Showing<'_>,
    index: &Index,
    within: Option<&[usize]>,
) -> (Vec<usize>, Vec<RowKey>) {
    let found = index.found(showing.search, within);
    let ordered = index.ordered(found.clone(), showing.sorting);
    let rows = pane.assemble(reading, showing, index, &ordered);
    (found, rows)
}
