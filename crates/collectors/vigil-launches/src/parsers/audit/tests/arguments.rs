use super::super::log::parse_audit_log;

use super::harness::{ONE_LAUNCH, read};

#[test]
fn arguments_written_in_hex_are_read_as_the_words_they_are() {
    let text = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=1000 exe="/bin/sh" key="vigil_exec""#,
        "\n",
        "type=EXECVE msg=audit(1757419203.412:3421): argc=3 a0=\"sh\" a1=2D63 a2=6563686F20686920796F75\n",
    );

    let reading = read(text);

    assert_eq!(
        reading.executions[0].arguments,
        vec!["sh", "-c", "echo hi you"]
    );
    assert!(!reading.executions[0].arguments_lossy);
}

#[test]
fn a_path_that_is_not_utf8_is_kept_and_marked_rather_than_dropped() {
    let text = concat!(
        "type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=1000 exe=2F746D702FFFFE6E63 key=\"vigil_exec\"\n",
        "type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0=FFFE\n",
    );

    let reading = read(text);

    let launch = &reading.executions[0];
    assert!(launch.executable_lossy, "the loss has to be visible");
    assert!(
        launch
            .executable
            .as_deref()
            .is_some_and(|path| path.starts_with("/tmp/")),
        "{:?}",
        launch.executable
    );
    assert!(launch.arguments_lossy);
}

#[test]
fn an_argument_written_in_pieces_is_put_back_together() {
    let text = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=1000 exe="/bin/sh" key="vigil_exec""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=2 a0="sh" a1_len=12 a1[0]="first" a1[1]="second""#,
        "\n",
    );

    let reading = read(text);

    assert_eq!(reading.executions[0].arguments, vec!["sh", "firstsecond"]);
}

#[test]
fn the_arguments_are_not_even_assembled_unless_they_were_asked_for() {
    let reading = parse_audit_log(ONE_LAUNCH.as_bytes(), false);

    assert!(reading.executions[0].arguments.is_empty());
}
