use vigil_model::{Killed, Killing, Snapshot};
use vigil_network::SocketView;

pub const READING: &str = "network";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub key: String,
    pub pid: u32,
    pub program: Option<String>,
    pub protocol: String,
    pub address: Option<String>,
    pub port: Option<u64>,
}

pub enum Aim {
    At(Target),
    Nowhere(Killed),
}

pub fn aim(key: &str, reading: Option<&Snapshot>, killing: Killing, ours: u32) -> Aim {
    let Some(reading) = reading else {
        return Aim::Nowhere(Killed::refused(
            key,
            "the agent has not read the sockets of this host yet",
        ));
    };
    let Some(item) = reading.items.get(key) else {
        return Aim::Nowhere(Killed::refused(
            key,
            "no longer in the reading the agent holds: the socket closed, or it was never \
             there",
        ));
    };

    let view = SocketView::new(item);
    if !view.is_socket() {
        return Aim::Nowhere(Killed::refused(
            key,
            "this row counts sockets, it is not one of them",
        ));
    }
    if killing == Killing::Destroy && view.transport() != "tcp" {
        return Aim::Nowhere(Killed::refused(
            key,
            format!(
                "a {} socket is not closed by destroying it: the kernel offers that for tcp \
                 alone. Stopping the process is what closes this one.",
                view.protocol()
            ),
        ));
    }
    if !view.owner_resolved() {
        return Aim::Nowhere(Killed::refused(
            key,
            "the process behind this socket was never resolved, so there is nothing to aim \
             at. The summary screen says which privilege is missing.",
        ));
    }
    let Some(pid) = view.pid() else {
        return Aim::Nowhere(Killed::refused(
            key,
            "the reading carries no pid for this socket",
        ));
    };
    if pid == 1 {
        return Aim::Nowhere(Killed::refused(
            key,
            "that is pid 1: signalling it stops the host, and this agent does not do that",
        ));
    }
    if pid == ours {
        return Aim::Nowhere(Killed::refused(
            key,
            "that is this agent: it does not kill itself on request",
        ));
    }

    Aim::At(Target {
        key: key.to_string(),
        pid,
        program: view.executable().map(str::to_string),
        protocol: view.protocol().to_string(),
        address: view.is_network_socket().then(|| view.address().to_string()),
        port: view.is_network_socket().then(|| view.port()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vigil_network::fixture;

    fn reading(key: &str, item: serde_json::Value) -> Snapshot {
        let mut snapshot = Snapshot::new(READING, "2026-09-14T10:00:00.000Z");
        snapshot.items.insert(key.to_string(), item);
        snapshot
    }

    fn refusal(aim: Aim) -> String {
        match aim {
            Aim::Nowhere(killed) => killed.said,
            Aim::At(target) => panic!("it aimed at {target:?}"),
        }
    }

    #[test]
    fn a_socket_that_left_the_reading_is_refused_by_name_rather_than_aimed_at_by_guess() {
        let said = refusal(aim(
            "tcp|0.0.0.0:4444",
            Some(&reading(
                "tcp|0.0.0.0:22",
                fixture::socket("0.0.0.0", 22, "/usr/sbin/sshd", "root"),
            )),
            Killing::Terminate,
            99,
        ));

        assert!(
            said.contains("no longer in the reading"),
            "between marking a row and confirming the kill the port can close and another \
             process take it; acting on the key alone would signal whatever holds it now: \
             {said}"
        );
    }

    #[test]
    fn pid_one_is_refused_because_signalling_it_is_stopping_the_host() {
        let mut item = fixture::socket("0.0.0.0", 22, "/usr/sbin/sshd", "root");
        item["process"]["pid"] = serde_json::json!(1);

        let said = refusal(aim(
            "tcp|0.0.0.0:22",
            Some(&reading("tcp|0.0.0.0:22", item)),
            Killing::Terminate,
            99,
        ));

        assert!(said.contains("pid 1"), "{said}");
    }

    #[test]
    fn the_agent_refuses_to_be_the_target_of_the_console_it_answers() {
        let mut item = fixture::socket("0.0.0.0", 22, "/usr/sbin/sshd", "root");
        item["process"]["pid"] = serde_json::json!(4242);

        let said = refusal(aim(
            "tcp|0.0.0.0:22",
            Some(&reading("tcp|0.0.0.0:22", item)),
            Killing::Kill,
            4242,
        ));

        assert!(said.contains("this agent"), "{said}");
    }

    #[test]
    fn a_socket_whose_owner_was_never_resolved_says_so_rather_than_failing_at_the_signal() {
        let said = refusal(aim(
            "tcp|0.0.0.0:22",
            Some(&reading(
                "tcp|0.0.0.0:22",
                fixture::socket_without_owner("0.0.0.0", 22),
            )),
            Killing::Terminate,
            99,
        ));

        assert!(said.contains("never resolved"), "{said}");
    }

    #[test]
    fn a_unix_socket_cannot_be_destroyed_and_is_told_what_would_close_it_instead() {
        let said = refusal(aim(
            "unix|/run/docker.sock",
            Some(&reading(
                "unix|/run/docker.sock",
                fixture::unix_socket("/run/docker.sock", "/usr/bin/dockerd", "root"),
            )),
            Killing::Destroy,
            99,
        ));

        assert!(
            said.contains("tcp alone") && said.contains("Stopping the process"),
            "an option that silently does nothing on half the rows teaches an operator that \
             the agent lies: {said}"
        );
    }

    #[test]
    fn the_same_unix_socket_is_a_target_when_the_ask_is_to_stop_the_process() {
        match aim(
            "unix|/run/docker.sock",
            Some(&reading(
                "unix|/run/docker.sock",
                fixture::unix_socket("/run/docker.sock", "/usr/bin/dockerd", "root"),
            )),
            Killing::Terminate,
            99,
        ) {
            Aim::At(target) => {
                assert_eq!(target.pid, 812);
                assert_eq!(target.program.as_deref(), Some("/usr/bin/dockerd"));
            }
            Aim::Nowhere(killed) => panic!("refused: {}", killed.said),
        }
    }
}
