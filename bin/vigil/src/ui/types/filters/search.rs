use ratatui::text::{Line, Span};

use crate::ui::Look;

const LONGEST: usize = 64;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Search {
    query: String,
    typing: bool,
    before: String,
}

impl Search {
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn typing(&self) -> bool {
        self.typing
    }

    pub fn holding_back(&self) -> bool {
        !self.query.is_empty()
    }

    pub fn matches(&self, haystack: &str) -> bool {
        if self.query.is_empty() {
            return true;
        }
        haystack.to_lowercase().contains(&self.query.to_lowercase())
    }

    pub fn start(&mut self) {
        self.typing = true;
        self.before = self.query.clone();
    }

    pub fn type_character(&mut self, character: char) {
        if self.query.chars().count() < LONGEST {
            self.query.push(character);
        }
    }

    pub fn erase(&mut self) {
        self.query.pop();
    }

    pub fn accept(&mut self) {
        self.typing = false;
    }

    pub fn abandon(&mut self) {
        self.typing = false;
        self.query = std::mem::take(&mut self.before);
    }

    pub fn clear(&mut self) {
        *self = Search::default();
    }

    pub fn line(&self, look: Look) -> Line<'static> {
        match self.typing {
            true => Line::from(vec![
                Span::styled(" search ", look.palette.heading()),
                Span::styled(self.query.clone(), look.palette.accent()),
                Span::styled("▏", look.palette.accent()),
                Span::styled(
                    "  every value the agent read about the row",
                    look.palette.quiet(),
                ),
            ]),
            false => Line::from(vec![
                Span::styled(" search ", look.palette.heading()),
                Span::styled(format!("{:?}", self.query), look.palette.accent()),
                Span::styled(
                    "  matched against every value the agent read",
                    look.palette.quiet(),
                ),
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::json;

    use super::*;
    use crate::ui::helpers::words::haystack::haystack;

    fn looking_for(wanted: &str) -> Search {
        let mut search = Search::default();
        search.start();
        for character in wanted.chars() {
            search.type_character(character);
        }
        search.accept();
        search
    }

    fn socket() -> Value {
        json!({
            "protocol": "tcp", "address": "0.0.0.0", "port": 4444, "uid": 33,
            "user": "www-data",
            "process": {"exe": "/tmp/.x/nc", "exe_deleted": true, "cmdline": "nc -l -p 4444"},
            "owner_resolved": true,
        })
    }

    #[test]
    fn an_empty_search_holds_nothing_back() {
        let search = Search::default();

        assert!(!search.holding_back());
        assert!(search.matches("anything at all"));
    }

    #[test]
    fn it_looks_at_every_value_the_agent_recorded_and_not_a_chosen_few() {
        let hay = haystack("tcp|0.0.0.0:4444", &socket());

        for wanted in [
            "0.0.0.0:4444",
            "tcp",
            "www-data",
            "/tmp/.x/nc",
            "nc -l -p 4444",
            "33",
            "4444",
        ] {
            assert!(looking_for(wanted).matches(&hay), "{wanted} was not found");
        }
        assert!(!looking_for("postgres").matches(&hay));
    }

    #[test]
    fn a_value_added_by_a_newer_collector_is_searchable_the_day_it_arrives() {
        let hay = haystack("tcp|0.0.0.0:22", &json!({"something_new": "wireguard"}));

        assert!(looking_for("wireguard").matches(&hay));
    }

    #[test]
    fn it_does_not_care_about_case_because_nobody_types_sha256_in_capitals() {
        let hay = haystack("sshkey|deploy", &json!({"fingerprint": "SHA256:AbC"}));

        assert!(looking_for("sha256:abc").matches(&hay));
        assert!(looking_for("SHA256:ABC").matches(&hay));
    }

    #[test]
    fn escape_out_of_the_box_puts_back_what_was_there_before_it_opened() {
        let mut search = looking_for("nginx");

        search.start();
        search.erase();
        search.type_character('!');
        search.abandon();

        assert_eq!(search.query(), "nginx");
        assert!(!search.typing());
    }

    #[test]
    fn a_pasted_log_line_does_not_run_off_the_side_of_the_box() {
        let mut search = Search::default();
        search.start();
        for _ in 0..500 {
            search.type_character('x');
        }

        assert!(search.query().chars().count() <= LONGEST);
    }
}
