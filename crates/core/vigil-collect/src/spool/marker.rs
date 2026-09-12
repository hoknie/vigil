const PREFIX: &str = "#vigil-spool ";

pub fn dropped_line(bytes: u64, ceiling: u64) -> String {
    format!(
        "{PREFIX}dropped {bytes} byte(s) of the oldest events: the spool reached its ceiling of {ceiling} bytes with nothing reading it\n"
    )
}

pub fn dropped_note(line: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(line).ok()?;
    text.strip_prefix(PREFIX)
        .map(|note| note.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_says_how_much_was_lost_and_why() {
        let line = dropped_line(8_388_608, 16_777_216);

        let note = dropped_note(line.as_bytes()).expect("recognised");
        assert!(note.contains("8388608"), "{note}");
        assert!(note.contains("16777216"), "{note}");
        assert!(!note.ends_with('\n'));
    }

    #[test]
    fn an_audit_record_is_not_mistaken_for_one_of_our_lines() {
        let record = br#"type=SYSCALL msg=audit(1757419203.412:3421): exe="/usr/bin/id""#;

        assert_eq!(dropped_note(record), None);
    }
}
