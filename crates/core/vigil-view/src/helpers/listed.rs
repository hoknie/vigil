use vigil_model::Snapshot;

use super::listing::listing;
use crate::{Index, Pane, RowKey, Rows, Showing};

pub fn listed(
    pane: &dyn Pane,
    reading: &Snapshot,
    showing: &Showing<'_>,
    index: &Index,
    within: Option<&[usize]>,
) -> (Vec<usize>, Vec<RowKey>) {
    let (found, assembled) = listing(pane, reading, showing, index, within);
    let rows = Rows::of(index, &assembled).to_vec();
    (found, rows)
}
