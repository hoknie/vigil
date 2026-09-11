use vigil_model::Snapshot;

use super::fields::flag;
use super::rows::{marks, objects, rows};
use super::showing::Showing;
use crate::ui::helpers::words::moment;
use crate::ui::{Program, Search, View};

pub(super) fn tally(view: &View, showing: &Showing<'_>, snapshot: &Snapshot, width: u16) -> String {
    let program = showing.program;
    let held = objects(view, program);
    let shown = rows(view, program, showing.search)
        .into_iter()
        .filter(|row| !row.mark)
        .count();

    let mut parts = vec![match showing.search.holding_back() {
        false => format!(
            "{held} {}, read at {}",
            program.things(held),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{shown} of {held} {}, read at {}",
            program.things(held),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];

    let unnamed = marks(view, program);
    if unnamed > 0 {
        parts.push(format!("{unnamed} row(s) about the reading itself"));
    }

    match program {
        Program::Running => {
            let gone = rows(view, program, &Search::default())
                .into_iter()
                .filter(|row| !row.mark && flag(row.item, "exe_deleted"))
                .count();
            if gone > 0 {
                parts.push(format!("{gone} whose file is gone"));
            }
        }
        Program::Launches => {
            parts.push("this list only grows".to_string());
        }
    }

    if showing.elsewhere > 0 {
        parts.push(format!(
            "{} other list(s) narrowed by a search of their own",
            showing.elsewhere
        ));
    }
    if let Some(reason) = view.collector_note(program.collector()) {
        parts.push(format!("incomplete: {reason}"));
    }

    let room = (width as usize).saturating_sub(1);
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
