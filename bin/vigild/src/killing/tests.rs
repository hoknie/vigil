use vigil_model::{Killing, Snapshot};
use vigil_processes::fixture;

use super::programs::{Aim, Program, aim, stop, written_out};

const NC: &str = "exec|/tmp/.x/nc|www-data";

fn reading() -> Snapshot {
    fixture::processes()
}

fn found(pids: &'static [u32]) -> impl Fn(&str, u32) -> Result<Vec<u32>, String> {
    move |_, _| Ok(pids.to_vec())
}

fn refusal(aim: Aim) -> String {
    match aim {
        Aim::Nowhere(killed) => killed.said,
        Aim::At(program) => panic!("it aimed at {program:?}"),
    }
}

#[test]
fn a_program_is_aimed_at_every_process_that_runs_it_as_that_account_now() {
    let asked: std::cell::RefCell<Option<(String, u32)>> = Default::default();
    let running = |executable: &str, uid: u32| {
        asked.replace(Some((executable.to_string(), uid)));
        Ok(vec![9001, 9002])
    };

    match aim(NC, Some(&reading()), Killing::Terminate, 99, &running) {
        Aim::At(program) => {
            assert_eq!(program.executable, "/tmp/.x/nc");
            assert_eq!(program.pids, vec![9001, 9002]);
        }
        Aim::Nowhere(killed) => panic!("refused: {}", killed.said),
    }
    assert_eq!(
        asked.take(),
        Some(("/tmp/.x/nc".to_string(), 33)),
        "the row is a program and an account; the processes are looked up now, because a \
         pid written into a reading thirty seconds ago may belong to someone else by the \
         time a person confirms"
    );
}

#[test]
fn a_program_that_left_the_reading_is_refused_by_name() {
    let said = refusal(aim(
        "exec|/usr/bin/gone|root",
        Some(&reading()),
        Killing::Terminate,
        99,
        &found(&[4242]),
    ));

    assert!(said.contains("no longer in the reading"), "{said}");
}

#[test]
fn the_row_about_the_reading_itself_is_not_a_program_to_stop() {
    let said = refusal(aim(
        "processes|unresolved",
        Some(&reading()),
        Killing::Kill,
        99,
        &found(&[4242]),
    ));

    assert!(said.contains("not a program"), "{said}");
}

#[test]
fn closing_a_socket_is_not_a_way_to_stop_a_program_and_the_refusal_says_what_is() {
    let said = refusal(aim(
        NC,
        Some(&reading()),
        Killing::Destroy,
        99,
        &found(&[9001]),
    ));

    assert!(said.contains("stopping its processes"), "{said}");
}

#[test]
fn a_program_nothing_runs_any_more_is_refused_rather_than_reported_as_stopped() {
    let said = refusal(aim(NC, Some(&reading()), Killing::Kill, 99, &found(&[])));

    assert!(said.contains("any more"), "{said}");
}

#[test]
fn a_program_that_pid_one_runs_is_refused_whole_and_not_stopped_around_pid_one() {
    let said = refusal(aim(
        "exec|/lib/systemd/systemd|root",
        Some(&reading()),
        Killing::Terminate,
        99,
        &found(&[1, 1200, 1300]),
    ));

    assert!(
        said.contains("pid 1"),
        "systemd as root is pid 1 and the user managers; signalling the rest and leaving \
         pid 1 is not what the row asked for, and signalling pid 1 stops the host: {said}"
    );
}

#[test]
fn the_program_this_agent_runs_as_is_refused_because_the_agent_does_not_kill_itself() {
    let said = refusal(aim(
        NC,
        Some(&reading()),
        Killing::Kill,
        9002,
        &found(&[9001, 9002]),
    ));

    assert!(said.contains("this agent"), "{said}");
}

#[test]
fn a_host_where_the_processes_cannot_be_looked_up_says_why_on_the_row() {
    let failing = |_: &str, _: u32| Err("/proc cannot be listed".to_string());

    let said = refusal(aim(NC, Some(&reading()), Killing::Kill, 99, &failing));

    assert!(said.contains("/proc cannot be listed"), "{said}");
}

#[test]
fn every_process_of_a_program_is_signalled_and_the_ones_that_were_not_are_named() {
    let program = Program {
        key: NC.into(),
        executable: "/tmp/.x/nc".into(),
        uid: 33,
        pids: vec![9001, 9002, 9003],
    };
    let signalling = |pid: u32, _: Killing| match pid {
        9002 => Err(format!("pid {pid} was gone before the signal reached it")),
        _ => Ok(format!("sent to {pid}")),
    };

    let killed = stop(&program, Killing::Terminate, &signalling);

    assert!(killed.done);
    assert_eq!(killed.pid, Some(9001));
    assert!(
        killed
            .said
            .contains("SIGTERM sent to 2 process(es): 9001, 9003"),
        "{}",
        killed.said
    );
    assert!(
        killed.said.contains("1 not signalled") && killed.said.contains("9002"),
        "an operator told that the program was stopped while one of its processes lives \
         on is an operator who stops looking: {}",
        killed.said
    );
}

#[test]
fn a_program_none_of_whose_processes_took_the_signal_is_a_refusal_and_not_a_success() {
    let program = Program {
        key: NC.into(),
        executable: "/tmp/.x/nc".into(),
        uid: 33,
        pids: vec![9001],
    };
    let signalling = |pid: u32, _: Killing| Err(format!("not allowed to signal pid {pid}"));

    let killed = stop(&program, Killing::Kill, &signalling);

    assert!(!killed.done);
    assert!(killed.said.contains("not allowed"), "{}", killed.said);
}

#[test]
fn a_program_with_many_processes_is_written_out_short_and_says_how_many_it_left_out() {
    let pids: Vec<u32> = (100..120).collect();

    assert_eq!(
        written_out(&pids),
        "100, 101, 102, 103, 104, 105, 106, 107 and 12 more"
    );
}
