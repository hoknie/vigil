use serde_json::Value;

pub struct LaunchView<'a>(&'a Value);

const WRITABLE_PATHS: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"];

impl<'a> LaunchView<'a> {
    pub fn new(value: &'a Value) -> Self {
        LaunchView(value)
    }

    pub fn is_launch(&self) -> bool {
        self.0.get("exe").is_some() && self.0.get("auid").is_some()
    }

    pub fn executable(&self) -> &'a str {
        self.0["exe"].as_str().unwrap_or("?")
    }

    pub fn user(&self) -> String {
        match self.0["user"].as_str() {
            Some(name) => name.to_string(),
            None => self.auid().to_string(),
        }
    }

    pub fn auid(&self) -> u64 {
        self.0["auid"].as_u64().unwrap_or(u64::MAX)
    }

    pub fn on_disk(&self) -> bool {
        self.0["exe_present"].as_bool().unwrap_or(true)
    }

    pub fn runs_from_writable_path(&self) -> bool {
        match self.0["writable_path"].as_bool() {
            Some(answer) => answer,
            None => WRITABLE_PATHS
                .iter()
                .any(|writable| self.executable().starts_with(writable)),
        }
    }

    pub fn audit_id(&self) -> Option<&'a str> {
        self.0["audit_id"].as_str()
    }

    pub fn arguments(&self) -> Option<&'a str> {
        self.0["arguments"].as_str()
    }

    pub fn arguments_redacted(&self) -> bool {
        self.0["arguments_redacted"].as_bool().unwrap_or(false)
    }

    pub fn executable_lossy(&self) -> bool {
        self.0["exe_lossy"].as_bool().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use vigil_rules::fixture as neighbours;

    #[test]
    fn an_item_of_another_collector_is_not_a_launch() {
        let program = neighbours::of_another_collector("exec|/usr/sbin/nginx|root");
        let socket = neighbours::of_another_collector("tcp|0.0.0.0:443");
        let account = neighbours::of_another_collector("account|deploy");

        assert!(!LaunchView::new(&program).is_launch());
        assert!(!LaunchView::new(&socket).is_launch());
        assert!(!LaunchView::new(&account).is_launch());
    }

    #[test]
    fn the_rows_saying_the_collector_could_not_see_something_are_not_launches() {
        let unnamed = serde_json::json!({"named": false, "reason": "…"});
        let capped = serde_json::json!({"named": false, "reason": "…"});

        assert!(!LaunchView::new(&unnamed).is_launch());
        assert!(!LaunchView::new(&capped).is_launch());
    }

    #[test]
    fn a_login_that_etc_passwd_does_not_name_is_shown_by_its_number() {
        let mut value = fixture::launch("alice", 1000, "/usr/bin/nc");
        value["user"] = Value::Null;

        assert_eq!(LaunchView::new(&value).user(), "1000");
    }
}
