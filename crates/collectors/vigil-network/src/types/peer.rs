use serde_json::Value;

use super::socket::WRITABLE_PATHS;

pub struct PeerView<'a> {
    key: &'a str,
    value: &'a Value,
}

impl<'a> PeerView<'a> {
    pub fn new(key: &'a str, value: &'a Value) -> Self {
        PeerView { key, value }
    }

    pub fn is_outbound(&self) -> bool {
        self.key.starts_with("outbound|") && self.value.get("remote_port").is_some()
    }

    pub fn protocol(&self) -> &'a str {
        self.value["protocol"].as_str().unwrap_or("?")
    }

    pub fn remote_address(&self) -> &'a str {
        self.value["remote_address"].as_str().unwrap_or("?")
    }

    pub fn remote_port(&self) -> u64 {
        self.value["remote_port"].as_u64().unwrap_or(0)
    }

    pub fn local_port(&self) -> u64 {
        self.value["local_port"].as_u64().unwrap_or(0)
    }

    pub fn program(&self) -> Option<&'a str> {
        self.value["program"].as_str()
    }

    pub fn program_deleted(&self) -> bool {
        self.value["program_deleted"].as_bool().unwrap_or(false)
    }

    pub fn owner_resolved(&self) -> bool {
        self.value["owner_resolved"].as_bool().unwrap_or(false)
    }

    pub fn user(&self) -> String {
        match self.value["user"].as_str() {
            Some(user) => user.to_string(),
            None => format!("uid {}", self.value["uid"].as_u64().unwrap_or(0)),
        }
    }

    pub fn suspicious_program(&self) -> bool {
        self.owner_resolved()
            && (self.program_deleted()
                || self
                    .program()
                    .is_some_and(|path| WRITABLE_PATHS.iter().any(|bad| path.starts_with(bad))))
    }
}
