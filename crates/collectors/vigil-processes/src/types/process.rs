use serde_json::Value;

pub struct ProcessView<'a>(&'a Value);

const WRITABLE_PATHS: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"];

const EXECUTES_WHAT_ARRIVES: &[&str] = &[
    "nginx",
    "apache2",
    "httpd",
    "lighttpd",
    "caddy",
    "haproxy",
    "php-fpm",
    "php",
    "uwsgi",
    "gunicorn",
    "unicorn",
    "puma",
    "passenger",
    "node",
    "java",
    "tomcat",
    "python",
    "ruby",
    "perl",
    "postgres",
    "mysqld",
    "mariadbd",
    "redis-server",
    "memcached",
];

const SHELLS: &[&str] = &[
    "sh", "bash", "dash", "ash", "zsh", "ksh", "csh", "tcsh", "fish", "busybox",
];

impl<'a> ProcessView<'a> {
    pub fn new(value: &'a Value) -> Self {
        ProcessView(value)
    }

    pub fn is_program(&self) -> bool {
        self.0.get("exe").is_some() && self.0["exe_resolved"].as_bool().unwrap_or(false)
    }

    pub fn executable(&self) -> &'a str {
        self.0["exe"].as_str().unwrap_or("?")
    }

    pub fn program_name(&self) -> &'a str {
        self.executable()
            .rsplit_once('/')
            .map(|(_, name)| name)
            .unwrap_or_else(|| self.executable())
    }

    pub fn executable_deleted(&self) -> bool {
        self.0["exe_deleted"].as_bool().unwrap_or(false)
    }

    pub fn user(&self) -> &'a str {
        self.0["user"].as_str().unwrap_or("?")
    }

    pub fn uid(&self) -> u64 {
        self.0["uid"].as_u64().unwrap_or(u64::MAX)
    }

    pub fn is_root(&self) -> bool {
        self.uid() == 0
    }

    pub fn command_line(&self) -> Option<&'a str> {
        self.0["cmdline"].as_str()
    }

    pub fn command_line_varies(&self) -> bool {
        self.0["cmdline_varies"].as_bool().unwrap_or(false)
    }

    pub fn command_line_redacted(&self) -> bool {
        self.0["cmdline_redacted"].as_bool().unwrap_or(false)
    }

    pub fn parents(&self) -> Vec<&'a str> {
        self.0["parents"]
            .as_array()
            .map(|values| values.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }

    pub fn runs_from_writable_path(&self) -> bool {
        match self.0["writable_path"].as_bool() {
            Some(answer) => answer,
            None => WRITABLE_PATHS
                .iter()
                .any(|writable| self.executable().starts_with(writable)),
        }
    }

    pub fn is_shell(&self) -> bool {
        SHELLS.contains(&self.program_name())
    }

    pub fn is_shell_under_a_service(&self) -> bool {
        self.is_shell()
            && self
                .parents()
                .iter()
                .any(|parent| Self::executes_what_arrives(parent))
    }

    pub fn executes_what_arrives(path: &str) -> bool {
        let name = path.rsplit_once('/').map(|(_, name)| name).unwrap_or(path);
        EXECUTES_WHAT_ARRIVES
            .iter()
            .any(|known| name == *known || name.starts_with(known))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use vigil_rules::fixture as neighbours;

    #[test]
    fn an_item_of_another_collector_is_not_a_program() {
        let socket = neighbours::of_another_collector("tcp|0.0.0.0:443");
        let account = neighbours::of_another_collector("account|deploy");
        let unit = neighbours::of_another_collector("unit|nginx.service");

        assert!(!ProcessView::new(&socket).is_program());
        assert!(!ProcessView::new(&account).is_program());
        assert!(!ProcessView::new(&unit).is_program());
    }

    #[test]
    fn the_row_saying_some_executables_could_not_be_read_is_not_a_program() {
        let marker = serde_json::json!({"exe_resolved": false, "reason": "…"});

        assert!(!ProcessView::new(&marker).is_program());
    }

    #[test]
    fn a_version_number_in_a_program_name_does_not_hide_it() {
        assert!(ProcessView::executes_what_arrives("/usr/sbin/php-fpm8.2"));
        assert!(ProcessView::executes_what_arrives("/usr/bin/python3.11"));
        assert!(ProcessView::executes_what_arrives("/usr/bin/node"));
        assert!(!ProcessView::executes_what_arrives(
            "/usr/lib/systemd/systemd"
        ));
        assert!(!ProcessView::executes_what_arrives("/usr/sbin/cron"));
    }

    #[test]
    fn a_shell_is_recognised_by_its_name_and_not_by_where_it_lives() {
        for path in ["/bin/sh", "/usr/bin/bash", "/bin/busybox", "/tmp/.x/dash"] {
            let value = fixture::program(path, "www-data", 33, &[]);
            assert!(ProcessView::new(&value).is_shell(), "{path}");
        }
        let curl = fixture::program("/usr/bin/curl", "root", 0, &[]);
        assert!(!ProcessView::new(&curl).is_shell());
    }
}
