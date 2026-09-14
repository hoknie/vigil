use super::super::log::{any_launch_carries_our_tag, record_is_read};

use super::harness::{ONE_LAUNCH, read};

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
fn a_rule_being_taken_is_not_a_record_of_somebody_running_something() {
    let taken = "type=CONFIG_CHANGE msg=audit(1757419203.500:3401): op=add_rule key=\"vigil_exec\" list=4 res=1\n";

    assert!(
        any_launch_carries_our_tag(ONE_LAUNCH.as_bytes()),
        "a launch the rule matched is what a scan of the tail is looking for"
    );
    assert!(
        !any_launch_carries_our_tag(taken.as_bytes()),
        "the tag in the record that loads the rule is the rule's own name, and reading it \
         as a launch tells an operator commands are being seen on a host where none has \
         been run yet"
    );
    assert!(!any_launch_carries_our_tag(
        b"type=SYSCALL msg=audit(1.0:1): key=\"audit-wazuh\"\n"
    ));
}
