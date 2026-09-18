use vigil_model::Severity;

use crate::ui::Column;

pub(super) const EVERY_KIND: &str = "every kind";

pub(super) fn offered(kinds: &[(String, usize)]) -> Vec<String> {
    let mut said: Vec<String> = Severity::KNOWN.iter().map(floor_named).collect();
    said.extend(Column::ALL.iter().map(|column| looking_in(*column)));
    if !kinds.is_empty() {
        said.push(EVERY_KIND.to_string());
        said.extend(kinds.iter().map(|(kind, held)| kind_named(kind, *held)));
    }
    said
}

pub(super) fn kind_named(kind: &str, held: usize) -> String {
    format!("{} ({held})", showing_only(kind))
}

pub(super) fn kind_chosen(said: &str) -> Option<Option<String>> {
    if said == EVERY_KIND {
        return Some(None);
    }
    let (kind, _) = said.strip_prefix("only ")?.rsplit_once(" (")?;
    Some(Some(kind.to_string()))
}

pub(super) fn floor_named(severity: &Severity) -> String {
    match severity {
        Severity::Info => "every severity".to_string(),
        other => format!("{} and above", other.as_str()),
    }
}

pub(super) fn looking_in(column: Column) -> String {
    format!("search in {}", column.name())
}

pub(super) fn showing_only(kind: &str) -> String {
    format!("only {kind}")
}
