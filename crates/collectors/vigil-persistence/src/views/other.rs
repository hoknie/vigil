use vigil_view::{Column, Width};

pub(super) fn columns(_wide: bool) -> Vec<Column> {
    vec![
        Column::new("KEY", Width::Least(24)),
        Column::new("WHAT THIS CONSOLE CAN SAY", Width::Share(2)),
    ]
}

pub(super) fn cells(key: &str) -> Vec<String> {
    vec![
        key.to_string(),
        "a kind of object this console does not know".to_string(),
    ]
}
