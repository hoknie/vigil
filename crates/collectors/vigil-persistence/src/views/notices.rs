use vigil_view::{Notice, Showing};

use crate::types::List;

pub(super) fn empty(list: List, showing: &Showing<'_>) -> Notice {
    if showing.holding_back() {
        return Notice::plain(format!("No {} matches {:?}.", list.thing(), showing.search)).saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }

    match showing.note {
        Some(reason) => {
            Notice::loud("Nothing here, and the reading is incomplete.").saying(format!(
                "Reason: {reason}. It may or may not be about this list; the summary screen \
                 carries the whole of it."
            ))
        }
        None => Notice::plain(format!("No {} in this reading.", list.thing())).saying(list.empty()),
    }
}
