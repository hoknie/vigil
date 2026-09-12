use vigil_model::{CollectorState, CollectorStatus};

use super::notices::{NO_SECTION, NOT_WATCHED};

pub(super) const NO_SCREEN: &str = "no screen draws it yet";
use super::row::Row;
use super::standing::Standing;
use crate::ui::helpers::words::moment;
use crate::ui::{Screen, View};

pub fn rows(view: &View) -> Vec<Row> {
    let mut rows: Vec<Row> = Screen::ALL
        .iter()
        .filter(|screen| screen.group() == crate::ui::Group::Reads)
        .map(|screen| section(view, *screen))
        .collect();
    rows.extend(strangers(view));
    rows.extend(
        Screen::ALL
            .iter()
            .filter(|screen| screen.group() == crate::ui::Group::Concludes)
            .map(|screen| section(view, *screen)),
    );
    rows
}

pub fn keys(view: &View) -> Vec<String> {
    rows(view).into_iter().map(|row| row.name).collect()
}

fn section(view: &View, screen: Screen) -> Row {
    Row {
        opens: Some(screen),
        number: screen.digit(),
        name: screen.name().to_string(),
        holds: screen.holds().to_string(),
        collector: screen.collectors().join(" · "),
        standing: match screen {
            Screen::Summary => answering(view),
            Screen::Findings => found(view),
            _ => reading(view, screen),
        },
    }
}

fn answering(view: &View) -> Standing {
    let losing = losing(view);

    Standing {
        read: view
            .as_of()
            .map(|when| moment::time_of_day(when).to_string()),
        unwell: view.stale().is_some() || losing.is_some(),
        note: losing,
        ..Standing::plain(match view.stale() {
            Some(_) => "not answering",
            None => "answering",
        })
    }
}

fn losing(view: &View) -> Option<String> {
    let buffers = view.status.as_ref()?.agent.buffers.as_ref()?;
    let said: Vec<String> = buffers
        .iter()
        .filter(|buffer| buffer.dropped_total > 0)
        .map(|buffer| {
            format!(
                "{} finding(s) waiting for {} were dropped at the ceiling of {}",
                buffer.dropped_total, buffer.receiver, buffer.pending_ceiling
            )
        })
        .collect();

    match said.is_empty() {
        true => None,
        false => Some(said.join(" · ")),
    }
}

fn found(view: &View) -> Standing {
    let severity = view
        .status
        .as_ref()
        .map(|status| &status.agent.findings.by_severity)
        .and_then(highest)
        .unwrap_or_else(|| "none".to_string());

    Standing {
        objects: Some(view.found.findings.len()),
        read: view
            .as_of()
            .map(|when| moment::time_of_day(when).to_string()),
        ..Standing::plain(severity)
    }
}

fn highest(by_severity: &std::collections::BTreeMap<String, u64>) -> Option<String> {
    for severity in ["critical", "high", "medium", "low", "info"] {
        if by_severity.get(severity).copied().unwrap_or(0) > 0 {
            return Some(severity.to_string());
        }
    }
    None
}

fn reading(view: &View, screen: Screen) -> Standing {
    let Some(status) = &view.status else {
        return Standing::plain("—");
    };

    let present: Vec<&CollectorStatus> = screen
        .collectors()
        .iter()
        .filter_map(|name| {
            status
                .agent
                .collectors
                .iter()
                .find(|collector| collector.name == *name)
        })
        .collect();

    if present.is_empty() {
        return Standing {
            note: Some(NOT_WATCHED.to_string()),
            ..Standing::plain("not watched")
        };
    }
    if present
        .iter()
        .all(|collector| collector.state == CollectorState::Off)
    {
        return Standing {
            note: reasons(&present),
            ..Standing::plain("off")
        };
    }

    let live: Vec<&&CollectorStatus> = present
        .iter()
        .filter(|collector| collector.state != CollectorState::Off)
        .collect();
    let unwell = live.iter().any(|collector| collector.state.is_trouble());

    Standing {
        state: worst(&live),
        objects: Some(live.iter().map(|collector| collector.items).sum()),
        read: live
            .iter()
            .filter_map(|collector| collector.last_run_at.as_deref())
            .max()
            .map(|when| moment::time_of_day(when).to_string()),
        note: reasons(&present),
        unwell,
    }
}

fn worst(live: &[&&CollectorStatus]) -> String {
    live.iter()
        .map(|collector| collector.state.clone())
        .find(|state| state.is_trouble())
        .unwrap_or(CollectorState::Ok)
        .as_str()
        .to_string()
}

fn reasons(present: &[&CollectorStatus]) -> Option<String> {
    let said: Vec<String> = present
        .iter()
        .filter_map(|collector| {
            collector
                .reason
                .as_deref()
                .map(|reason| match present.len() > 1 {
                    true => format!("{}: {reason}", collector.name),
                    false => reason.to_string(),
                })
        })
        .collect();
    match said.is_empty() {
        true => None,
        false => Some(said.join(" · ")),
    }
}

fn strangers(view: &View) -> Vec<Row> {
    let Some(status) = &view.status else {
        return Vec::new();
    };

    status
        .agent
        .collectors
        .iter()
        .filter(|collector| Screen::holding(&collector.name).is_none())
        .map(|collector| Row {
            opens: None,
            number: None,
            name: collector.name.clone(),
            holds: NO_SCREEN.to_string(),
            collector: collector.name.clone(),
            standing: Standing {
                objects: match collector.state == CollectorState::Off {
                    true => None,
                    false => Some(collector.items),
                },
                read: collector
                    .last_run_at
                    .as_deref()
                    .map(|when| moment::time_of_day(when).to_string()),
                note: Some(NO_SECTION.to_string()),
                unwell: collector.state.is_trouble(),
                state: collector.state.as_str().to_string(),
            },
        })
        .collect()
}
