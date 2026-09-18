use vigil_view::Pane;

use super::notices::{missing, said};
use crate::ui::{Notice, Reading, View};

pub(super) fn of_the_group(
    panes: &[Box<dyn Pane>],
    shown: &[usize],
    group: Option<&str>,
) -> Vec<usize> {
    if group.is_none() {
        return shown.to_vec();
    }

    shown
        .iter()
        .copied()
        .filter(|index| {
            panes
                .get(*index)
                .is_some_and(|pane| pane.belongs_to() == group)
        })
        .collect()
}

pub(super) fn named(groups: &[&'static str]) -> Vec<(usize, String)> {
    groups
        .iter()
        .enumerate()
        .map(|(at, group)| (at, (*group).to_string()))
        .collect()
}

pub(super) fn chosen(groups: &[&'static str], group: Option<&str>) -> usize {
    groups
        .iter()
        .position(|named| Some(*named) == group)
        .unwrap_or(0)
}

pub(super) fn nothing_to_show(
    view: &View,
    panes: &[Box<dyn Pane>],
    group: Option<&str>,
) -> Option<Notice> {
    group?;
    let pane = panes.iter().find(|pane| pane.belongs_to() == group)?;

    if let Some(notice) = missing(view, pane.as_ref()) {
        return Some(notice);
    }
    let everything = vigil_view::Showing::default();
    let explained = match view.reading(pane.reads()) {
        Reading::Taken(reading) => pane.why_nothing_is_listed(reading, &everything),
        _ => None,
    };

    Some(said(
        explained
            .or_else(|| pane.nothing_in_the_reading())
            .unwrap_or_else(|| pane.empty(&everything)),
    ))
}
