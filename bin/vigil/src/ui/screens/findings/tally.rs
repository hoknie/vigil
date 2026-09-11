use ratatui::layout::Rect;
use vigil_model::StoreStatus;

use crate::ui::{Filter, View};

pub(super) fn tally(view: &View, filter: &Filter, footer: Rect) -> String {
    let shown = filter.passing(&view.found.findings).len();
    let mut parts = vec![format!(
        "{shown} shown of {} held{} · cap {}",
        view.found.findings.len(),
        match filter.holding_back() {
            true => format!(" · {}", filter.describe()),
            false => String::new(),
        },
        view.found.capacity,
    )];

    if view.found.dropped > 0 {
        parts.push(format!(
            "{} more in the journal{}",
            view.found.dropped,
            match journal(view).and_then(|store| store.journal_path.clone()) {
                Some(path) => format!(": {path}"),
                None => String::new(),
            }
        ));
    }
    if let Some(damaged) = journal(view)
        .map(|store| store.damaged)
        .filter(|it| *it > 0)
    {
        parts.push(format!("{damaged} unreadable line(s) skipped at start"));
    }

    let room = (footer.width as usize).saturating_sub(1);
    let mut line = String::new();
    for part in parts {
        let next = match line.is_empty() {
            true => part,
            false => format!("{line} · {part}"),
        };
        if next.chars().count() > room {
            break;
        }
        line = next;
    }
    format!(" {line}")
}

pub(super) fn journal(view: &View) -> Option<&StoreStatus> {
    view.status.as_ref()?.agent.store.as_ref()
}
