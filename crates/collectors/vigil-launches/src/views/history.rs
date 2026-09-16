use serde_json::Value;
use vigil_view::Piece;

use super::fields::marked;
use super::launches::{executable, last_run, runs, when, who};
use crate::helpers::{Moment, moment};
use crate::parsers::{RAN_AT, RECENT, RECENT_RUNS};

const FINDING_ONE: &str = "Where no command line was kept, ausearch -a with the number beside \
                           the run finds it in the host's audit log.";

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

    let mut kept: Vec<Run<'_>> = item
        .get(RECENT)
        .and_then(Value::as_array)
        .map(|runs| runs.iter().filter_map(Run::of).collect())
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

    kept.sort_by(|left, right| (right.moment, right.id).cmp(&(left.moment, left.id)));
    said.push(Piece::heading(format!(
        "THE LAST {} RUN(S), NEWEST FIRST",
        kept.len()
    )));
    for (at, run) in kept.iter().enumerate() {
        said.push(Piece::field(
            (at + 1).to_string(),
            match run.moment {
                Some(moment) => format!("{} · {}", when(moment), run.said()),
                None => format!(
                    "a run this build cannot read the moment of · {}",
                    run.said()
                ),
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
    if kept
        .iter()
        .any(|run| run.arguments.is_none() && !run.redacted)
    {
        said.push(Piece::text(FINDING_ONE));
    }
    said
}

struct Run<'a> {
    moment: Option<Moment>,
    id: &'a str,
    arguments: Option<&'a str>,
    redacted: bool,
}

impl<'a> Run<'a> {
    fn of(said: &'a Value) -> Option<Run<'a>> {
        if let Some(id) = said.as_str() {
            return Some(Run {
                moment: moment(id),
                id,
                arguments: None,
                redacted: false,
            });
        }
        let id = said.get(RAN_AT)?.as_str()?;
        Some(Run {
            moment: moment(id),
            id,
            arguments: said.get("arguments").and_then(Value::as_str),
            redacted: said
                .get("arguments_redacted")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
    }

    fn said(&self) -> String {
        match (self.arguments, self.redacted) {
            (Some(arguments), _) => arguments.to_string(),
            (None, true) => {
                "arguments hidden on this host before they were written down".to_string()
            }
            (None, false) => format!("ausearch -a {}", serial(self.id)),
        }
    }
}

fn serial(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id)
}
