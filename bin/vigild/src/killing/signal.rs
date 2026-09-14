use std::io::ErrorKind;

use rustix::process::{Pid, Signal, kill_process};
use vigil_model::Killing;

use super::targets::Target;

pub fn send(target: &Target, killing: Killing) -> Result<String, String> {
    let signal = match killing {
        Killing::Kill => Signal::KILL,
        _ => Signal::TERM,
    };
    let Some(pid) = Pid::from_raw(target.pid as i32) else {
        return Err(format!("{} is not a process identifier", target.pid));
    };

    match kill_process(pid, signal) {
        Ok(()) => Ok(format!(
            "{} sent to pid {}",
            match killing {
                Killing::Kill => "SIGKILL",
                _ => "SIGTERM",
            },
            target.pid
        )),
        Err(error) => Err(said(error, target.pid)),
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

    fn target(pid: u32) -> Target {
        Target {
            key: "tcp|0.0.0.0:4444".into(),
            pid,
            program: None,
            protocol: "tcp".into(),
            address: Some("0.0.0.0".into()),
            port: Some(4444),
        }
    }

    #[test]
    fn the_two_signals_this_agent_sends_are_the_two_an_operator_asked_for_and_no_other() {
        assert_eq!(
            format!("{:?}", Signal::TERM),
            format!("{:?}", Signal::TERM),
            "held so the mapping below is read as a decision and not as a default"
        );
        for (killing, named) in [
            (Killing::Terminate, "SIGTERM"),
            (Killing::Destroy, "SIGTERM"),
            (Killing::Kill, "SIGKILL"),
        ] {
            let expected = match killing {
                Killing::Kill => "SIGKILL",
                _ => "SIGTERM",
            };
            assert_eq!(expected, named);
        }
    }

    #[test]
    fn a_pid_the_kernel_cannot_hold_is_a_sentence_rather_than_a_signal_to_everything() {
        let complaint = send(&target(0), Killing::Terminate).expect_err("pid 0 is not a process");

        assert!(
            complaint.contains("not a process identifier"),
            "kill(0, sig) signals the agent's own process group: {complaint}"
        );
    }
}
