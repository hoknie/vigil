use crate::ui::{Filter, View};

pub(super) fn tally(view: &View, filter: &Filter, shown: usize) -> String {
    format!(
        " {shown} shown of {} held{} · cap {} · {} dropped",
        view.found.findings.len(),
        match filter.holding_back() {
            true => format!(" · {}", filter.describe()),
            false => String::new(),
        },
        view.found.capacity,
        view.found.dropped,
    )
}
