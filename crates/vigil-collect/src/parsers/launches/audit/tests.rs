use super::log::{TRAILING_LINES, parse_audit_log, record_is_read};
use super::reading::AuditReading;

const ONE_LAUNCH: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 a0=55d2b5e4a2c0 a1=55d2b5e4a340 a2=55d2b5e4a3a0 a3=8 items=2 ppid=2143 pid=2170 auid=1000 uid=1000 gid=1000 euid=1000 suid=1000 fsuid=1000 egid=1000 sgid=1000 fsgid=1000 tty=pts0 ses=3 comm="nc" exe="/usr/bin/nc.openbsd" subj=unconfined key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419203.412:3421): argc=3 a0="nc" a1="-l" a2="4444""#,
    "\n",
    r#"type=CWD msg=audit(1757419203.412:3421): cwd="/home/alice""#,
    "\n",
    r#"type=PATH msg=audit(1757419203.412:3421): item=0 name="/usr/bin/nc" inode=1441806 dev=fd:01 mode=0100755 ouid=0 ogid=0 rdev=00:00 nametype=NORMAL cap_fp=0 cap_fi=0 cap_fe=0 cap_fver=0"#,
    "\n",
    "type=PROCTITLE msg=audit(1757419203.412:3421): proctitle=6E63002D6C0034343434\n",
);

fn read(text: &str) -> AuditReading {
    parse_audit_log(text.as_bytes(), true)
}

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
fn a_failed_execve_is_not_a_launch() {
    let text = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=no exit=-2 auid=1000 exe="/usr/bin/bash" key="vigil_exec""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="nosuchthing""#,
        "\n",
    );

    assert!(read(text).executions.is_empty());
}

#[test]
fn somebody_elses_audit_rule_is_not_read() {
    let text = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=1000 exe="/usr/bin/nc" key="their_own_rule""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="nc""#,
        "\n",
    );

    assert!(read(text).executions.is_empty(), "we read only our own key");
}

#[test]
fn a_record_carrying_several_keys_at_once_is_still_ours() {
    let both = "766967696C5F657865630174686569725F72756C65";
    let text = format!(
        concat!(
            r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=1000 exe="/usr/bin/nc" key={}"#,
            "\n",
            r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="nc""#,
            "\n",
        ),
        both
    );

    assert_eq!(read(&text).executions.len(), 1);
}

#[test]
fn everything_that_is_not_an_execve_is_read_past() {
    let text = concat!(
        "type=USER_LOGIN msg=audit(1757419203.412:3400): pid=1 uid=0 auid=1000 ses=3 msg='op=login id=1000 exe=\"/usr/sbin/sshd\" res=success'\n",
        "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"vigil_exec\" list=4 res=1\n",
    );

    let reading = read(text);

    assert!(reading.executions.is_empty());
    assert_eq!(reading.unnamed, 0);
}

#[test]
fn the_kernel_saying_it_took_our_rule_is_how_this_agent_learns_the_rule_is_standing() {
    let taken = "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"vigil_exec\" list=4 res=1\n";

    assert!(
        read(taken).rule_loaded,
        "a host where the rule is loaded and nobody has run anything looks exactly like a \
         host where it was never loaded, and this record is the only thing in the stream \
         that tells the two apart"
    );
    assert!(!read("").rule_loaded);
}

#[test]
fn a_rule_that_is_not_ours_taken_or_ours_refused_is_not_our_rule_standing() {
    for line in [
        "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"audit-wazuh\" list=4 res=1\n",
        "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"vigil_exec\" list=4 res=0\n",
        "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=remove_rule key=\"vigil_exec\" list=4 res=1\n",
        "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule list=4 res=1\n",
    ] {
        assert!(
            !read(line).rule_loaded,
            "somebody else's rule, a refused one and a removed one each say nothing about \
             ours: {line}"
        );
    }
}

#[test]
fn the_record_that_says_the_rule_was_taken_is_one_the_plugin_puts_in_the_spool() {
    let taken = "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"vigil_exec\" list=4 res=1\n";

    assert!(
        record_is_read(taken.as_bytes()),
        "the plugin writes only what the collector reads, so a record the collector needs \
         and the plugin drops is a signal that never arrives on a host using the plugin"
    );
    assert!(!record_is_read(
        b"type=PROCTITLE msg=audit(1757419203.412:3421): proctitle=2F62696E2F7368\n"
    ));
}

#[test]
fn a_rule_record_holds_no_execution_back_and_does_not_rewind_the_cursor() {
    let text = concat!(
        "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"vigil_exec\" list=4 res=1\n",
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=0 exe="/usr/bin/id" key="vigil_exec""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="id""#,
        "\n",
    );

    let reading = read(text);

    assert_eq!(reading.executions.len(), 1);
    assert!(reading.rule_loaded);
    assert_eq!(
        reading.consumed,
        text.len(),
        "a record that is no event of its own must not look like an event waiting for its \
         second half, or the cursor never moves past it"
    );
}

#[test]
fn a_launch_in_a_root_session_is_read_because_root_has_loginuid_zero() {
    let text = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=0 exe="/usr/bin/id" key="vigil_exec""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="id""#,
        "\n",
    );

    let reading = read(text);

    assert_eq!(reading.executions.len(), 1);
    assert_eq!(
        reading.executions[0].auid,
        Some(0),
        "most servers are administered as root, and a reading that drops loginuid 0 is empty \
         on all of them"
    );
}

#[test]
fn a_launch_whose_program_cannot_be_named_is_counted_rather_than_dropped() {
    let text = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): success=yes auid=1000 key="vigil_exec""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="mystery""#,
        "\n",
    );

    let reading = read(text);

    assert!(reading.executions.is_empty());
    assert_eq!(
        reading.unnamed, 1,
        "silence here reads as 'nobody ran anything'"
    );
}

#[test]
fn the_arguments_are_not_even_assembled_unless_they_were_asked_for() {
    let reading = parse_audit_log(ONE_LAUNCH.as_bytes(), false);

    assert!(reading.executions[0].arguments.is_empty());
}
