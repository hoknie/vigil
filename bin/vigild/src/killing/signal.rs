use std::io::ErrorKind;

use rustix::process::{Pid, Signal, kill_process};
use vigil_model::Killing;

pub type StillThere<'a> = &'a dyn Fn(u32) -> bool;

pub fn send(pid: u32, killing: Killing, still_there: StillThere<'_>) -> Result<String, String> {
    let Some(process) = i32::try_from(pid)
        .ok()
        .filter(|raw| *raw > 0)
        .and_then(Pid::from_raw)
    else {
        return Err(format!("{pid} is not a process identifier"));
    };

    delivered(process, pid, signal(killing), still_there)
        .map(|()| format!("{} sent to pid {pid}", named(killing)))
}

pub fn named(killing: Killing) -> &'static str {
    match killing {
        Killing::Kill => "SIGKILL",
        _ => "SIGTERM",
    }
}

fn signal(killing: Killing) -> Signal {
    match killing {
        Killing::Kill => Signal::KILL,
        _ => Signal::TERM,
    }
}

#[cfg(target_os = "linux")]
fn delivered(
    process: Pid,
    pid: u32,
    signal: Signal,
    still_there: StillThere<'_>,
) -> Result<(), String> {
    use rustix::process::{PidfdFlags, pidfd_open, pidfd_send_signal};

    match pidfd_open(process, PidfdFlags::empty()) {
        Ok(handle) => {
            if !still_there(pid) {
                return Err(moved(pid));
            }
            pidfd_send_signal(&handle, signal).map_err(|error| said(error, pid))
        }
        Err(rustix::io::Errno::NOSYS) => checked_then_killed(process, pid, signal, still_there),
        Err(error) => Err(said(error, pid)),
    }
}

#[cfg(not(target_os = "linux"))]
fn delivered(
    process: Pid,
    pid: u32,
    signal: Signal,
    still_there: StillThere<'_>,
) -> Result<(), String> {
    checked_then_killed(process, pid, signal, still_there)
}

fn checked_then_killed(
    process: Pid,
    pid: u32,
    signal: Signal,
    still_there: StillThere<'_>,
) -> Result<(), String> {
    if !still_there(pid) {
        return Err(moved(pid));
    }
    kill_process(process, signal).map_err(|error| said(error, pid))
}

fn moved(pid: u32) -> String {
    format!(
        "pid {pid} no longer runs what was marked: that process ended and the number belongs \
         to another one now, which was left alone"
    )
}

fn said(error: rustix::io::Errno, pid: u32) -> String {
    match std::io::Error::from(error).kind() {
        ErrorKind::NotFound => format!("pid {pid} was gone before the signal reached it"),
        ErrorKind::PermissionDenied => format!(
            "the agent is not allowed to signal pid {pid}: it runs under a capability set \
             that does not include CAP_KILL, or the process is in another namespace"
        ),
        _ => format!("pid {pid} was not signalled: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_signals_this_agent_sends_are_the_two_an_operator_asked_for_and_no_other() {
        assert_eq!(named(Killing::Terminate), "SIGTERM");
        assert_eq!(named(Killing::Kill), "SIGKILL");
        assert_eq!(signal(Killing::Terminate), Signal::TERM);
        assert_eq!(signal(Killing::Kill), Signal::KILL);
    }

    #[test]
    fn a_pid_the_kernel_cannot_hold_is_a_sentence_rather_than_a_signal_to_everything() {
        for pid in [0, u32::MAX] {
            let complaint = send(pid, Killing::Terminate, &|_| true).expect_err("not a process");

            assert!(
                complaint.contains("not a process identifier"),
                "kill(0, sig) signals the agent's own process group and kill(-1, sig) every \
                 process it may signal: {complaint}"
            );
        }
    }

    #[test]
    fn a_pid_that_no_longer_runs_what_was_marked_is_left_alone_and_the_refusal_says_so() {
        let complaint = send(std::process::id(), Killing::Terminate, &|_| false)
            .expect_err("the check said this is not the marked process");

        assert!(
            complaint.contains("no longer runs what was marked"),
            "a pid read from /proc a moment ago can be reused by the time the signal goes; \
             the process is asked again after it is held, and a stranger is not signalled: \
             {complaint}"
        );
    }
}
