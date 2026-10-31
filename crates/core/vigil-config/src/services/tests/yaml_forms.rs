use std::path::Path;

use crate::{Block, blocks_in, silenced};

fn read(text: &str) -> Result<Vec<(String, bool, Option<u32>)>, String> {
    blocks_in(Path::new("collectors.yaml"), text).map(|blocks| {
        blocks
            .into_iter()
            .map(
                |Block {
                     name,
                     enabled,
                     schedule,
                     ..
                 }| (name, enabled, schedule),
            )
            .collect()
    })
}

fn suppression(entry: &str) -> Result<(Option<String>, Option<String>), String> {
    let held = silenced(&format!("suppressions:\n  - {entry}\n"))?;
    Ok((held[0].finding_key.clone(), held[0].until.clone()))
}

#[test]
fn yes_in_a_boolean_key_is_refused_rather_than_read_as_true() {
    for word in ["yes", "on", "no", "off"] {
        let refused = read(&format!("network:\n  enabled: {word}\n"))
            .expect_err("a word that YAML 1.1 read as a boolean is text here");
        assert!(refused.contains("enabled is true or false"), "{refused}");
    }
}

#[test]
fn an_octal_looking_number_reads_as_it_does_today() {
    assert_eq!(
        read("network:\n  schedule: 0600\n"),
        Ok(vec![("network".to_string(), true, Some(600))]),
        "a leading zero is a decimal number, not octal"
    );
    assert_eq!(
        read("network:\n  schedule: 0o600\n"),
        Ok(vec![("network".to_string(), true, Some(384))])
    );
    assert_eq!(
        suppression("finding_key: 0600\n    reason: r"),
        Ok((Some("0600".to_string()), None)),
        "a key written as digits is kept as it was written"
    );
}

#[test]
fn a_tilde_and_an_empty_value_are_absent() {
    for until in ["~", ""] {
        assert_eq!(
            suppression(&format!(
                "finding_key: a\n    reason: r\n    until: {until}"
            )),
            Ok((Some("a".to_string()), None)),
            "until: {until:?}"
        );
        assert!(
            read(&format!("network:\n  schedule: {until}\n")).is_err(),
            "a schedule of nothing is not a period: {until:?}"
        );
    }
}

#[test]
fn an_underscore_in_a_number_reads_as_it_does_today() {
    let refused = read("network:\n  schedule: 1_000\n").expect_err("text, not a number");

    assert!(refused.contains("\"1_000\" is not a period"), "{refused}");
}

#[test]
fn an_unquoted_date_and_moment_in_until_are_text() {
    for until in ["2026-12-31", "2026-12-31T00:00:00Z"] {
        assert_eq!(
            suppression(&format!(
                "finding_key: a\n    reason: r\n    until: {until}"
            )),
            Ok((
                Some("a".to_string()),
                Some("2026-12-31T00:00:00.000Z".to_string())
            )),
            "until: {until}"
        );
    }
}

#[test]
fn a_duplicate_key_is_refused() {
    let refused = read("network:\n  schedule: 30\nnetwork:\n  schedule: 60\n")
        .expect_err("two blocks of one name in one mapping");
    assert!(refused.contains("duplicate"), "{refused}");

    let refused = suppression("finding_key: a\n    reason: r\n    reason: s")
        .expect_err("one entry saying its reason twice");
    assert!(refused.contains("duplicate"), "{refused}");
}

#[test]
fn a_merge_key_reads_as_it_does_today() {
    let refused =
        silenced("base: &base\n  reason: r\nsuppressions:\n  - finding_key: a\n    <<: *base\n")
            .expect_err("the merge key is not expanded");

    assert!(refused.contains("unknown field"), "{refused}");
    assert!(refused.contains("<<"), "{refused}");
}

#[test]
fn several_documents_in_one_collectors_file_are_read_in_order() {
    assert_eq!(
        read("network:\n  schedule: 30\n---\nusers:\n  schedule: 60\n"),
        Ok(vec![
            ("network".to_string(), true, Some(30)),
            ("users".to_string(), true, Some(60)),
        ])
    );
}

#[test]
fn an_empty_document_is_skipped() {
    assert_eq!(
        read("---\n---\nnetwork:\n  schedule: 30\n"),
        Ok(vec![("network".to_string(), true, Some(30))])
    );
    assert_eq!(read(""), Ok(Vec::new()));
}
