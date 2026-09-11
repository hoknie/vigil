pub fn switched_off_reason(name: &str) -> String {
    format!(
        "not named in `collectors:`, so nothing is watching {}: switched off, not failing",
        vigil_collect::subject_of_collector(name).unwrap_or("what it watches")
    )
}
