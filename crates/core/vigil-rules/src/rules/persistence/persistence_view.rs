use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Unit,
    Timer,
    Cron,
    Module,
    Script,
    Preload,
}

pub struct PersistenceView<'a> {
    key: &'a str,
    value: &'a Value,
}

const WRITABLE_PATHS: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"];

impl<'a> PersistenceView<'a> {
    pub fn new(key: &'a str, value: &'a Value) -> Self {
        PersistenceView { key, value }
    }

    pub fn family(&self) -> Option<Family> {
        match self.key.split_once('|')?.0 {
            "unit" => Some(Family::Unit),
            "timer" => Some(Family::Timer),
            "cron" => Some(Family::Cron),
            "module" => Some(Family::Module),
            "script" => Some(Family::Script),
            "preload" => Some(Family::Preload),
            _ => None,
        }
    }

    pub fn is(&self, family: Family) -> bool {
        self.family() == Some(family)
    }

    pub fn name(&self) -> &'a str {
        self.value["name"].as_str().unwrap_or("?")
    }

    pub fn path(&self) -> &'a str {
        self.value["path"].as_str().unwrap_or("?")
    }

    pub fn description(&self) -> Option<&'a str> {
        self.value["description"].as_str()
    }

    pub fn readable(&self) -> bool {
        self.value["readable"].as_bool().unwrap_or(true)
    }

    pub fn commands(&self) -> String {
        self.value["commands"]
            .as_array()
            .map(|commands| {
                commands
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default()
    }

    pub fn commands_redacted(&self) -> bool {
        self.value["commands_redacted"].as_bool().unwrap_or(false)
    }

    pub fn run_as(&self) -> &'a str {
        self.value["run_as"].as_str().unwrap_or("root")
    }

    pub fn schedule(&self) -> String {
        if let Some(schedule) = self.value["schedule"].as_str() {
            return schedule.to_string();
        }
        let calendar: Vec<&str> = self.value["on_calendar"]
            .as_array()
            .map(|values| values.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        match (calendar.is_empty(), self.value["on_boot"].as_str()) {
            (false, _) => calendar.join(", "),
            (true, Some(on_boot)) => format!("{on_boot} after boot"),
            (true, None) => "on request".to_string(),
        }
    }

    pub fn activates(&self) -> &'a str {
        self.value["activates"].as_str().unwrap_or("?")
    }

    pub fn command(&self) -> &'a str {
        self.value["command"].as_str().unwrap_or("?")
    }

    pub fn command_redacted(&self) -> bool {
        self.value["command_redacted"].as_bool().unwrap_or(false)
    }

    pub fn user(&self) -> &'a str {
        self.value["user"].as_str().unwrap_or("?")
    }

    pub fn source(&self) -> &'a str {
        self.value["source"].as_str().unwrap_or("?")
    }

    pub fn script_family(&self) -> &'a str {
        self.value["family"].as_str().unwrap_or("profile")
    }

    pub fn present(&self) -> bool {
        self.value["present"].as_bool().unwrap_or(true)
    }

    pub fn digest(&self) -> Option<&'a str> {
        self.value["sha256"].as_str()
    }

    pub fn mode(&self) -> &'a str {
        self.value["mode"].as_str().unwrap_or("?")
    }

    pub fn owner(&self) -> (u64, u64) {
        (
            self.value["uid"].as_u64().unwrap_or(0),
            self.value["gid"].as_u64().unwrap_or(0),
        )
    }

    pub fn entries(&self) -> Vec<&'a str> {
        self.value["entries"]
            .as_array()
            .map(|values| values.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }

    pub fn runs_from_writable_path(&self) -> bool {
        let commands = match self.family() {
            Some(Family::Cron) => self.command().to_string(),
            _ => self.commands(),
        };
        commands
            .split_whitespace()
            .any(|word| WRITABLE_PATHS.iter().any(|bad| word.starts_with(bad)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    #[test]
    fn an_item_of_another_collector_belongs_to_no_family_here() {
        let socket = fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root");
        let account = fixture::account("deploy", 1000, "/bin/bash");

        assert_eq!(
            PersistenceView::new("tcp|0.0.0.0:443", &socket).family(),
            None
        );
        assert_eq!(
            PersistenceView::new("account|deploy", &account).family(),
            None
        );
    }

    #[test]
    fn the_row_saying_the_module_list_could_not_be_read_is_not_a_module() {
        let marker = serde_json::json!({"readable": false, "reason": "…"});

        assert_eq!(
            PersistenceView::new("modules|unreadable", &marker).family(),
            None,
            "a rule about modules must not read the row that says there are none to read"
        );
    }

    #[test]
    fn a_command_hidden_in_the_middle_of_a_cron_line_is_still_found() {
        let job = fixture::cron_job(
            "/etc/crontab",
            "root",
            "@reboot",
            "cd / && /tmp/.x/implant --quiet",
        );
        let view = PersistenceView::new("cron|/etc/crontab|root|x", &job);

        assert!(view.runs_from_writable_path());
    }

    #[test]
    fn a_timer_with_no_calendar_says_what_it_does_have() {
        let timer = fixture::timer("boot.timer", &[], Some("15min"));

        assert_eq!(
            PersistenceView::new("timer|boot.timer", &timer).schedule(),
            "15min after boot"
        );
    }
}
