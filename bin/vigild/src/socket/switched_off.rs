pub fn switched_off_reason(name: &str) -> String {
    format!(
        "not named in `collectors:`, so nothing is watching {}: switched off, not failing",
        crate::modules::subject_of(name).unwrap_or("what it watches")
    )
}
