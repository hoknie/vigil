use std::path::Path;
use std::process::Command;

use super::targets::Target;

const WHERE_SS_LIVES: &[&str] = &["/usr/sbin/ss", "/sbin/ss", "/usr/bin/ss", "/bin/ss"];

pub fn destroy(target: &Target) -> Result<String, String> {
    let Some(tool) = found() else {
        return Err(format!(
            "closing a socket without stopping its process is done by ss, which is not on \
             this host (looked in {})",
            WHERE_SS_LIVES.join(", ")
        ));
    };
    let Some(filter) = filter(target) else {
        return Err("this socket has no address to name to ss".to_string());
    };

    let run = Command::new(tool)
        .args(["-K", "--no-header", "--tcp", "state", "all", &filter])
        .output()
        .map_err(|error| format!("ss could not be run: {error}"))?;

    let said = String::from_utf8_lossy(&run.stderr).trim().to_string();
    if !run.status.success() {
        return Err(match said.is_empty() {
            true => format!("ss refused, and said nothing ({})", run.status),
            false => format!("ss refused: {said}"),
        });
    }

    let closed = String::from_utf8_lossy(&run.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    Ok(match closed {
        0 => "ss closed nothing: the kernel here has no SOCK_DESTROY for this socket, or \
              nothing was connected to it"
            .to_string(),
        _ => format!("ss closed {closed} socket(s) on this address"),
    })
}

fn found() -> Option<&'static str> {
    WHERE_SS_LIVES
        .iter()
        .copied()
        .find(|path| Path::new(path).exists())
}

fn filter(target: &Target) -> Option<String> {
    let port = target.port?;
    match target.address.as_deref() {
        Some("0.0.0.0") | Some("::") | None => Some(format!("sport = :{port}")),
        Some(address) => Some(format!("sport = :{port} and src {address}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(address: &str, port: u64) -> Target {
        Target {
            key: format!("tcp|{address}:{port}"),
            pid: 30211,
            program: None,
            protocol: "tcp".into(),
            address: Some(address.to_string()),
            port: Some(port),
        }
    }

    #[test]
    fn a_socket_bound_to_one_address_is_named_by_that_address_and_not_by_the_port_alone() {
        assert_eq!(
            filter(&target("127.0.0.1", 5432)).expect("a filter"),
            "sport = :5432 and src 127.0.0.1",
            "the host can be listening on the same port on two addresses, and closing both \
             when the operator marked one is closing something nobody asked about"
        );
    }

    #[test]
    fn a_socket_bound_to_every_address_is_named_by_the_port_because_that_is_what_it_is() {
        assert_eq!(
            filter(&target("0.0.0.0", 8080)).expect("a filter"),
            "sport = :8080"
        );
        assert_eq!(
            filter(&target("::", 8080)).expect("a filter"),
            "sport = :8080"
        );
    }

    #[test]
    fn a_host_without_ss_is_told_so_by_name_rather_than_left_with_a_silent_failure() {
        if found().is_some() {
            return;
        }

        let complaint = destroy(&target("0.0.0.0", 8080)).expect_err("there is no ss here");

        assert!(complaint.contains("/usr/sbin/ss"), "{complaint}");
    }
}
