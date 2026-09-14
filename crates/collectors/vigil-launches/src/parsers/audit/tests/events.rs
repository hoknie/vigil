use super::super::log::TRAILING_LINES;
use super::harness::{ONE_LAUNCH, read};

#[test]
fn the_five_lines_of_one_launch_become_one_execution() {
    let reading = read(ONE_LAUNCH);

    assert_eq!(reading.executions.len(), 1);
    let launch = &reading.executions[0];
    assert_eq!(launch.executable.as_deref(), Some("/usr/bin/nc.openbsd"));
    assert_eq!(launch.auid, Some(1000));
    assert_eq!(launch.arguments, vec!["nc", "-l", "4444"]);
    assert_eq!(launch.id, "1757419203.412:3421");
    assert_eq!(reading.consumed, ONE_LAUNCH.len());
}

#[test]
fn the_lines_of_an_event_are_joined_by_their_identifier_whatever_order_they_arrive_in() {
    let shuffled: String = {
        let mut lines: Vec<&str> = ONE_LAUNCH.lines().collect();
        lines.reverse();
        lines.join("\n") + "\n"
    };

    let reading = read(&shuffled);

    assert_eq!(reading.executions.len(), 1);
    assert_eq!(
        reading.executions[0].executable.as_deref(),
        Some("/usr/bin/nc.openbsd")
    );
    assert_eq!(reading.executions[0].arguments, vec!["nc", "-l", "4444"]);
}

#[test]
fn two_events_written_into_each_other_are_still_two() {
    let interleaved = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=1000 uid=1000 exe="/usr/bin/nc" key="vigil_exec""#,
        "\n",
        r#"type=SYSCALL msg=audit(1757419203.999:3422): success=yes auid=1001 uid=1001 exe="/usr/bin/curl" key="vigil_exec""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="nc""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.999:3422): argc=1 a0="curl""#,
        "\n",
    );

    let reading = read(interleaved);

    assert_eq!(reading.executions.len(), 2);
    assert_eq!(reading.executions[0].auid, Some(1000));
    assert_eq!(reading.executions[1].auid, Some(1001));
    assert_eq!(reading.consumed, interleaved.len());
}

#[test]
fn an_event_still_being_written_is_kept_for_the_next_reading_instead_of_reported_short() {
    let text = format!(
        "{ONE_LAUNCH}{}",
        r#"type=SYSCALL msg=audit(1757419204.000:3422): success=yes auid=1000 exe="/usr/bin/id" key="vigil_exec""#,
    ) + "\n";

    let reading = read(&text);

    assert_eq!(
        reading.executions.len(),
        1,
        "the half-written one is not it"
    );
    assert_eq!(
        reading.consumed,
        ONE_LAUNCH.len(),
        "the offset must stop where the unfinished event starts"
    );
}

#[test]
fn a_line_with_no_newline_yet_is_neither_parsed_nor_consumed() {
    let text = format!("{ONE_LAUNCH}type=SYSCALL msg=audit(1757419204.0");

    let reading = read(&text);

    assert_eq!(reading.executions.len(), 1);
    assert_eq!(reading.consumed, ONE_LAUNCH.len());
}

#[test]
fn a_torn_event_at_the_start_of_a_chunk_does_not_stop_the_reader_for_ever() {
    let mut text =
        String::from("type=EXECVE msg=audit(1757419200.000:3000): argc=1 a0=\"orphan\"\n");
    for line in 0..TRAILING_LINES + 2 {
        text.push_str(&format!("type=EOE msg=audit(1757419201.{line:03}:3100)\n"));
    }
    text.push_str(ONE_LAUNCH);

    let reading = read(&text);

    assert_eq!(reading.executions.len(), 1);
    assert_eq!(reading.consumed, text.len(), "the reader must move on");
}
