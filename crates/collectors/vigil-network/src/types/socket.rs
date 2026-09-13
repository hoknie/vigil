use serde_json::Value;

pub struct SocketView<'a>(&'a Value);

const WRITABLE_PATHS: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"];

impl<'a> SocketView<'a> {
    pub fn new(value: &'a Value) -> Self {
        SocketView(value)
    }

    pub fn is_socket(&self) -> bool {
        self.0.get("protocol").is_some()
            && (self.0.get("port").is_some() || self.0.get("path").is_some())
    }

    pub fn is_network_socket(&self) -> bool {
        self.0.get("protocol").is_some() && self.0.get("port").is_some()
    }

    pub fn protocol(&self) -> &'a str {
        self.0["protocol"].as_str().unwrap_or("?")
    }

    pub fn transport(&self) -> &'a str {
        match self.protocol() {
            "tcp" | "tcp6" => "tcp",
            "udp" | "udp6" => "udp",
            other => other,
        }
    }

    pub fn address(&self) -> &'a str {
        self.0["address"].as_str().unwrap_or("?")
    }

    pub fn port(&self) -> u64 {
        self.0["port"].as_u64().unwrap_or(0)
    }

    pub fn path(&self) -> Option<&'a str> {
        self.0["path"].as_str()
    }

    pub fn endpoint(&self) -> String {
        match self.path() {
            Some(path) => path.to_string(),
            None => format!("{}:{}", self.address(), self.port()),
        }
    }

    pub fn user(&self) -> Option<&'a str> {
        self.0["user"].as_str()
    }

    pub fn executable(&self) -> Option<&'a str> {
        self.0["process"]["exe"].as_str()
    }

    pub fn executable_deleted(&self) -> bool {
        self.0["process"]["exe_deleted"].as_bool().unwrap_or(false)
    }

    pub fn command_line(&self) -> Option<&'a str> {
        self.0["process"]["cmdline"].as_str()
    }

    pub fn command_line_redacted(&self) -> bool {
        self.0["process"]["cmdline_redacted"]
            .as_bool()
            .unwrap_or(false)
    }

    pub fn owner_resolved(&self) -> bool {
        self.0["owner_resolved"].as_bool().unwrap_or(false)
    }

    pub fn world_reachable(&self) -> bool {
        self.is_network_socket()
            && !matches!(self.address(), "127.0.0.1" | "::1" | "::ffff:127.0.0.1")
    }

    pub fn loopback_only(&self) -> bool {
        self.is_network_socket() && !self.world_reachable()
    }

    pub fn suspicious_executable(&self) -> bool {
        if self.executable_deleted() {
            return true;
        }
        match self.executable() {
            Some(path) => WRITABLE_PATHS.iter().any(|bad| path.starts_with(bad)),
            None => false,
        }
    }

    pub fn describe_owner(&self) -> String {
        match (self.executable(), self.user()) {
            (Some(exe), Some(user)) => format!("{exe} as {user}"),
            (Some(exe), None) => exe.to_string(),
            (None, Some(user)) => format!("an unidentified process as {user}"),
            (None, None) => "an unidentified process".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[test]
    fn a_unix_socket_is_a_socket_and_is_never_reachable_from_the_network() {
        let value = fixture::unix_socket("/run/docker.sock", "/usr/bin/dockerd", "root");
        let view = SocketView::new(&value);

        assert!(view.is_socket());
        assert!(
            !view.is_network_socket(),
            "which is what every rule about a port checks"
        );
        assert!(
            !view.world_reachable(),
            "it has no address, and asking the address alone answers yes"
        );
        assert_eq!(view.endpoint(), "/run/docker.sock");
    }

    #[test]
    fn the_row_counting_the_sockets_with_no_name_is_not_a_socket() {
        let value = serde_json::json!({"protocol": "unix", "count": 3, "owner_resolved": false});

        assert!(!SocketView::new(&value).is_socket());
    }

    #[test]
    fn the_address_family_is_collapsed_away_so_a_dual_stack_bind_is_one_service() {
        let v4 = fixture::socket("0.0.0.0", 8080, "/usr/sbin/nginx", "root");
        let mut v6 = fixture::socket("::", 8080, "/usr/sbin/nginx", "root");
        v6["protocol"] = serde_json::json!("tcp6");

        assert_eq!(SocketView::new(&v4).transport(), "tcp");
        assert_eq!(SocketView::new(&v6).transport(), "tcp");
    }
}
