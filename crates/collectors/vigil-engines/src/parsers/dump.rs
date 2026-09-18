use crate::types::Dump;

pub fn parse_dump(bytes: &[u8]) -> Result<Dump, String> {
    let dump: Dump = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;

    if crate::types::Engine::named(&dump.engine).is_none() {
        return Err(format!(
            "the document names the engine {:?}, which this build does not read",
            dump.engine
        ));
    }
    if dump.state != crate::types::PRESENT && dump.state != crate::types::ABSENT {
        return Err(format!(
            "the document says the engine is {:?}, and the only two answers to that are {:?} \
             and {:?}",
            dump.state,
            crate::types::PRESENT,
            crate::types::ABSENT
        ));
    }

    Ok(dump)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WRITTEN: &str = r#"{
        "engine": "docker",
        "state": "present",
        "taken_at": "2026-09-17T09:00:00.000Z",
        "written_by": "/usr/sbin/vigil-container-dump",
        "deadline_seconds": 10,
        "program": "/usr/bin/docker",
        "asked": {
            "image": {"state": "answered", "arguments": ["image", "ls"], "milliseconds": 42,
                      "printed": "{\"ID\":\"one\"}\n", "truncated": false}
        }
    }"#;

    #[test]
    fn the_document_the_dump_writes_is_the_document_this_collector_reads() {
        let dump = parse_dump(WRITTEN.as_bytes()).expect("reads");

        assert!(dump.on_this_host());
        assert_eq!(dump.program.as_deref(), Some("/usr/bin/docker"));
        assert!(dump.answer("image").expect("asked for").answered());
        assert!(dump.unanswered().is_empty());
    }

    #[test]
    fn a_document_that_is_not_json_at_all_is_a_named_refusal_and_not_an_empty_reading() {
        let refusal = parse_dump(b"docker: command not found\n").expect_err("must not be read");

        assert!(!refusal.is_empty());
    }

    #[test]
    fn a_document_about_an_engine_this_build_does_not_read_is_refused_by_name() {
        let other = WRITTEN.replace("\"docker\"", "\"containerd\"");

        let refusal = parse_dump(other.as_bytes()).expect_err("must not be read");

        assert!(refusal.contains("containerd"), "{refusal}");
    }

    #[test]
    fn a_document_whose_engine_is_neither_present_nor_absent_is_refused_rather_than_guessed() {
        let neither = WRITTEN.replace("\"present\"", "\"maybe\"");

        let refusal = parse_dump(neither.as_bytes()).expect_err("must not be read");

        assert!(refusal.contains("maybe"), "{refusal}");
        assert!(refusal.contains("absent"), "{refusal}");
    }

    #[test]
    fn a_command_that_did_not_answer_is_carried_through_to_the_reader_by_name() {
        let broken = WRITTEN.replace("\"answered\"", "\"failed\"").replace(
            "\"truncated\": false",
            "\"truncated\": false, \"why\": \"Cannot connect to the Docker daemon\"",
        );

        let dump = parse_dump(broken.as_bytes()).expect("reads");

        assert_eq!(
            dump.unanswered(),
            vec!["image Cannot connect to the Docker daemon".to_string()],
            "a command that failed and a command that answered nothing look the same in the \
             reading, and only the document says which it was"
        );
    }
}
