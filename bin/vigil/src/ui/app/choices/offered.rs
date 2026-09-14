use vigil_model::Severity;

use crate::ui::Column;

pub(super) fn offered() -> Vec<String> {
    let mut said: Vec<String> = Severity::KNOWN.iter().map(floor_named).collect();
    said.extend(Column::ALL.iter().map(|column| looking_in(*column)));
    said
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
