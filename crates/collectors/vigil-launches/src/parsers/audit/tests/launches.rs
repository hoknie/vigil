use super::harness::read;

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
