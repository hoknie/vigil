use vigil_model::Snapshot;

use crate::{Assembled, Index, Pane, Showing};

pub fn listing(
    pane: &dyn Pane,
    reading: &Snapshot,
    showing: &Showing<'_>,
    index: &Index,
    within: Option<&[usize]>,
) -> (Vec<usize>, Assembled) {
    let found = match index.narrowed(showing.only) {
        None => index.found(showing.search, within),
        Some(narrowed) if showing.search.is_empty() => narrowed,
        Some(narrowed) => {
            let among = match within {
                Some(within) if within.len() < narrowed.len() => within,
                _ => narrowed.as_slice(),
            };
            index
                .found(showing.search, Some(among))
                .into_iter()
                .filter(|at| narrowed.binary_search(at).is_ok())
                .collect()
        }
    };
    let ordered = index.ordered(found.clone(), showing.sorting);
    let assembled = pane.assemble(reading, showing, index, ordered);
    (found, assembled)
}
