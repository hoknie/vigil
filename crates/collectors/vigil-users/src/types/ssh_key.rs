use serde_json::Value;

pub struct SshKeyView<'a>(&'a Value);

impl<'a> SshKeyView<'a> {
    pub fn new(value: &'a Value) -> Self {
        SshKeyView(value)
    }

    pub fn user(&self) -> &str {
        self.0["user"].as_str().unwrap_or("?")
    }

    pub fn uid(&self) -> u64 {
        self.0["uid"].as_u64().unwrap_or(u64::MAX)
    }

    pub fn algorithm(&self) -> &str {
        self.0["algorithm"].as_str().unwrap_or("?")
    }

    pub fn fingerprint(&self) -> &str {
        self.0["fingerprint"].as_str().unwrap_or("?")
    }

    pub fn comment(&self) -> Option<&str> {
        self.0["comment"].as_str()
    }

    pub fn options(&self) -> Option<&str> {
        self.0["options"].as_str()
    }

    pub fn source(&self) -> &str {
        self.0["source"].as_str().unwrap_or("?")
    }

    pub fn readable(&self) -> bool {
        self.0["readable"].as_bool().unwrap_or(false)
    }

    pub fn describe(&self) -> String {
        match self.comment() {
            Some(comment) => format!("{} {} ({comment})", self.algorithm(), self.fingerprint()),
            None => format!("{} {}", self.algorithm(), self.fingerprint()),
        }
    }
}
