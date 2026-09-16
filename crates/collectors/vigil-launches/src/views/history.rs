use serde_json::Value;
use vigil_view::Piece;

use super::fields::marked;
use super::launches::{executable, last_run, runs, when, who};
use crate::helpers::{Moment, moment};
use crate::parsers::{RECENT, RECENT_RUNS};

const FINDING_ONE: &str = "ausearch -a <the number after the colon> finds a run in the host's \
                           audit log, with its arguments and the process that started it.";

pub(super) fn history(key: &str, item: &Value) -> Vec<Piece> {
    if marked(key) {
        return vec![
            Piece::Warning("THE READING ITSELF".to_string()),
            Piece::Blank,
            Piece::text(
                "This row is about the reading, not about a program: nothing ran under it, \
                 so it has no runs to list.",
            ),
        ];
    }

    let mut said = vec![
        Piece::title("RUNS OF", executable(item)),
        Piece::Blank,
        Piece::field("who", who(item)),
        Piece::field(
            "runs",
            format!(
                "{} since this agent began reading the audit records",
                runs(item)
            ),
        ),
        Piece::Blank,
    ];

    let mut kept: Vec<(Option<Moment>, &str)> = item
        .get(RECENT)
        .and_then(Value::as_array)
        .map(|ids| {
            ids.iter()
                .filter_map(Value::as_str)
                .map(|id| (moment(id), id))
                .collect()
        })
        .unwrap_or_default();
    if kept.is_empty() {
        said.push(Piece::text(
            "No moment of a run is kept for this row: an older agent wrote it, and counted \
             runs without keeping when they happened. Its next run starts the list.",
        ));
        if let Some(last) = last_run(item) {
            said.push(Piece::field("last run", when(last)));
        }
        return said;
    }

    kept.sort_by(|left, right| right.cmp(left));
    said.push(Piece::heading(format!(
        "THE LAST {} RUN(S), NEWEST FIRST",
        kept.len()
    )));
    for (at, (moment, id)) in kept.iter().enumerate() {
        said.push(Piece::field(
            (at + 1).to_string(),
            match moment {
                Some(moment) => format!("{} · audit id {id}", when(*moment)),
                None => format!("audit id {id}, a moment this build cannot read"),
            },
        ));
    }
    said.push(Piece::Blank);

    let older = runs(item).saturating_sub(kept.len() as u64);
    if older > 0 {
        said.push(Piece::text(format!(
            "This agent keeps the moments of the last {RECENT_RUNS} runs of a row. The {older} \
             run(s) before them are in the host's audit log for as long as it holds them."
        )));
    }
    said.push(Piece::text(FINDING_ONE));
    said
}
