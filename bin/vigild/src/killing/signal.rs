use std::io::ErrorKind;

use rustix::process::{Pid, Signal, kill_process};
use vigil_model::Killing;

pub fn send(pid: u32, killing: Killing) -> Result<String, String> {
    let Some(process) = i32::try_from(pid)
        .ok()
        .filter(|raw| *raw > 0)
        .and_then(Pid::from_raw)
    else {
        return Err(format!("{pid} is not a process identifier"));
    };

    match kill_process(process, signal(killing)) {
        Ok(()) => Ok(format!("{} sent to pid {pid}", named(killing))),
        Err(error) => Err(said(error, pid)),
    }
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
            let complaint = send(pid, Killing::Terminate).expect_err("not a process");

            assert!(
                complaint.contains("not a process identifier"),
                "kill(0, sig) signals the agent's own process group and kill(-1, sig) every \
                 process it may signal: {complaint}"
            );
        }
    }
}
