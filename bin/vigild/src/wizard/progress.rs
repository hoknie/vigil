use super::Surveyed;
use super::plan::Planned;
use super::prose::wrap;

pub const WIDTH: usize = 80;

const MARGIN: usize = 2;

const WORD: usize = 6;

const VERB: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Write,
    Unchanged,
    Replace,
    Keep,
    SetAside,
    Leave,
    Nothing,
}

impl Action {
    pub fn of(planned: &Planned, there: Option<&str>, force: bool) -> Action {
        if there.is_some() && there == planned.text.as_deref() {
            return Action::Unchanged;
        }
        match (planned.text.is_some(), there.is_some(), force) {
            (true, false, _) => Action::Write,
            (true, true, true) => Action::Replace,
            (true, true, false) => Action::Keep,
            (false, true, true) => Action::SetAside,
            (false, true, false) => Action::Leave,
            (false, false, _) => Action::Nothing,
        }
    }

    pub fn touches_nothing_that_is_there(self) -> bool {
        matches!(self, Action::Keep | Action::Leave)
    }

    fn verb(self, dry_run: bool) -> &'static str {
        match (self, dry_run) {
            (Action::Write, false) => "wrote",
            (Action::Write, true) => "write",
            (Action::Unchanged, _) => "unchanged",
            (Action::Replace, false) => "replaced",
            (Action::Replace, true) => "replace",
            (Action::Keep, false) => "kept",
            (Action::Keep, true) => "keep",
            (Action::SetAside, _) => "set aside",
            (Action::Leave, false) => "left",
            (Action::Leave, true) => "leave",
            (Action::Nothing, _) => "",
        }
    }
}

pub fn survey(survey: &[Surveyed]) -> Vec<String> {
    let width = survey
        .iter()
        .map(|collector| collector.name.chars().count())
        .max()
        .unwrap_or(0);
    survey
        .iter()
        .flat_map(|collector| surveyed(collector, width))
        .collect()
}

pub fn said(action: Action, planned: &Planned, dry_run: bool) -> Vec<String> {
    if action == Action::Nothing {
        return Vec::new();
    }
    let path = planned.path.display().to_string();
    let previous = format!("{}.previous", file_name(&planned.path));
    let holds = planned.holds.join(" and ");
    let detail = match (action, dry_run) {
        (Action::Write | Action::Unchanged | Action::Nothing, _) => None,
        (Action::Replace, false) => Some(format!("what was there is kept as {previous}")),
        (Action::Replace, true) => Some(format!("what is there would be kept as {previous}")),
        (Action::Keep, _) => Some(format!(
            "it is already there, and without --force it is not touched; --force replaces it \
             and keeps what was there as {previous}"
        )),
        (Action::SetAside, false) => Some(format!(
            "{holds} cannot run here, so the file is kept as {previous} and read no more"
        )),
        (Action::SetAside, true) => Some(format!(
            "{holds} cannot run here, so the file would be kept as {previous} and read no more"
        )),
        (Action::Leave, _) => Some(format!(
            "{holds} cannot run here, and without --force the file is not touched: the \
             daemon will report it unavailable"
        )),
    };

    let mut lines = vec![format!(
        "{}{:<width$}{path}",
        " ".repeat(MARGIN),
        action.verb(dry_run),
        width = VERB
    )];
    if let Some(detail) = detail {
        let indent = MARGIN + VERB;
        for line in wrap(&detail, WIDTH - indent) {
            lines.push(format!("{}{line}", " ".repeat(indent)));
        }
    }
    lines
}

fn surveyed(collector: &Surveyed, width: usize) -> Vec<String> {
    let head = format!(
        "{}{:<word$}{:<width$}",
        " ".repeat(MARGIN),
        collector.word(),
        collector.name,
        word = WORD,
        width = width
    );
    let Some(reason) = collector.reason() else {
        return vec![head.trim_end().to_string()];
    };
    let indent = head.chars().count() + 2;
    wrap(reason, WIDTH.saturating_sub(indent).max(20))
        .into_iter()
        .enumerate()
        .map(|(index, line)| match index {
            0 => format!("{head}  {line}"),
            _ => format!("{}{line}", " ".repeat(indent)),
        })
        .collect()
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}
