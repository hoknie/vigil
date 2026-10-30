use serde::{Deserialize, Serialize};

use crate::helpers::pattern::{matches, matches_everything};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Suppression {
    pub finding_key: Option<String>,
    pub finding_key_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finding_key_match: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    pub kind: Option<String>,
    pub reason: String,
    pub until: Option<String>,
}

enum Key<'a> {
    Exact(&'a str),
    Prefix(&'a str),
    Pattern(&'a str),
    Any,
}

impl Suppression {
    pub fn validate(&self) -> Result<(), String> {
        let named: Vec<&str> = [
            ("finding_key", &self.finding_key),
            ("finding_key_prefix", &self.finding_key_prefix),
            ("finding_key_match", &self.finding_key_match),
        ]
        .into_iter()
        .filter(|(_, value)| value.is_some())
        .map(|(name, _)| name)
        .collect();

        if named.is_empty() && self.kind.is_none() && self.program.is_none() {
            return Err(
                "a suppression needs finding_key, finding_key_prefix, finding_key_match, kind or program; as written it silences nothing"
                    .into(),
            );
        }
        if named.len() > 1 {
            return Err(format!(
                "a suppression names the key in two ways at once ({}); keep one",
                named.join(", ")
            ));
        }
        if let Some(pattern) = &self.finding_key_match
            && matches_everything(pattern)
        {
            return Err(format!(
                "finding_key_match \"{pattern}\" matches every key; name the key, or silence by kind instead"
            ));
        }
        if let Some(pattern) = &self.program
            && matches_everything(pattern)
        {
            return Err(format!(
                "program \"{pattern}\" matches every program; name the program, or silence by kind instead"
            ));
        }
        if self.reason.trim().is_empty() {
            return Err("a suppression needs a reason".into());
        }
        Ok(())
    }

    pub fn covers(&self, finding_key: &str, kind: &str, program: Option<&str>, now: &str) -> bool {
        if let Some(until) = &self.until
            && until.as_str() <= now
        {
            return false;
        }
        if let Some(only) = &self.kind
            && only != kind
        {
            return false;
        }
        if let Some(pattern) = &self.program
            && !program.is_some_and(|program| Self::runs(pattern, program))
        {
            return false;
        }
        match self.key() {
            Key::Exact(exact) => exact == finding_key,
            Key::Prefix(prefix) => finding_key.starts_with(prefix),
            Key::Pattern(pattern) => matches(pattern, finding_key),
            Key::Any => self.kind.is_some() || self.program.is_some(),
        }
    }

    pub fn says_the_same_as(&self, other: &Suppression) -> bool {
        self.finding_key == other.finding_key
            && self.finding_key_prefix == other.finding_key_prefix
            && self.finding_key_match == other.finding_key_match
            && self.program == other.program
            && self.kind == other.kind
    }

    pub fn describe(&self) -> String {
        let what = self.object();
        match &self.kind {
            Some(kind) => format!("{what} ({kind}) — {}", self.reason),
            None => format!("{what} — {}", self.reason),
        }
    }

    pub fn object(&self) -> String {
        let key = match self.key() {
            Key::Exact(exact) => Some(exact.to_string()),
            Key::Prefix(prefix) => Some(format!("{prefix}*")),
            Key::Pattern(pattern) => Some(pattern.to_string()),
            Key::Any => None,
        };
        match (&self.program, key) {
            (Some(program), Some(key)) => format!("program {program} · {key}"),
            (Some(program), None) => format!("program {program}"),
            (None, Some(key)) => key,
            (None, None) => "anywhere".to_string(),
        }
    }

    fn key(&self) -> Key<'_> {
        match (
            &self.finding_key,
            &self.finding_key_prefix,
            &self.finding_key_match,
        ) {
            (Some(exact), _, _) => Key::Exact(exact),
            (_, Some(prefix), _) => Key::Prefix(prefix),
            (_, _, Some(pattern)) => Key::Pattern(pattern),
            _ => Key::Any,
        }
    }

    fn runs(pattern: &str, program: &str) -> bool {
        match pattern.contains('/') {
            true => matches(pattern, program),
            false => matches(pattern, program.rsplit('/').next().unwrap_or(program)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suppression(key: Option<&str>, prefix: Option<&str>) -> Suppression {
        Suppression {
            finding_key: key.map(str::to_string),
            finding_key_prefix: prefix.map(str::to_string),
            reason: "expected on this host".into(),
            ..Suppression::default()
        }
    }

    #[test]
    fn an_entry_that_silences_nothing_is_refused_rather_than_believed() {
        assert!(suppression(None, None).validate().is_err());
        assert!(
            suppression(Some("a"), Some("b")).validate().is_err(),
            "two ways of naming the same thing make what it covers unreadable"
        );
        assert!(suppression(Some("a"), None).validate().is_ok());
    }

    #[test]
    fn a_kind_on_its_own_covers_that_statement_anywhere_on_the_host() {
        let entry = Suppression {
            kind: Some("port.listen.removed".into()),
            reason: "we do not care when services stop here".into(),
            ..Suppression::default()
        };

        assert!(entry.validate().is_ok());
        assert!(entry.covers(
            "port.listen|tcp|0.0.0.0:80",
            "port.listen.removed",
            None,
            "2026-09-09T10:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:80",
            "port.listen.new",
            None,
            "2026-09-09T10:00:00.000Z"
        ));
    }

    #[test]
    fn two_entries_that_silence_the_same_statement_are_one_whatever_their_reasons_say() {
        let first = suppression(Some("port.listen|tcp|0.0.0.0:8080"), None);
        let mut second = first.clone();
        second.reason = "written again by somebody else".into();
        second.until = Some("2026-12-31T00:00:00.000Z".into());

        assert!(first.says_the_same_as(&second));

        second.kind = Some("port.listen.new".into());
        assert!(
            !first.says_the_same_as(&second),
            "silencing one kind about an object is a narrower statement than silencing the object"
        );
        assert!(
            !first.says_the_same_as(&suppression(None, Some("port.listen|tcp|0.0.0.0:8080"))),
            "a prefix and an exact key spelled alike cover different things"
        );
    }

    #[test]
    fn a_suppression_without_a_reason_is_refused() {
        let mut entry = suppression(Some("port.listen|tcp|0.0.0.0:8080"), None);
        entry.reason = "  ".into();

        assert!(entry.validate().is_err());
    }

    #[test]
    fn an_exact_key_covers_that_object_and_nothing_next_to_it() {
        let entry = suppression(Some("port.listen|tcp|0.0.0.0:8080"), None);

        assert!(entry.covers(
            "port.listen|tcp|0.0.0.0:8080",
            "port.listen.new",
            None,
            "2026-09-09T10:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:8081",
            "port.listen.new",
            None,
            "2026-09-09T10:00:00.000Z"
        ));
    }

    #[test]
    fn a_prefix_covers_a_range_and_a_kind_narrows_it() {
        let mut entry = suppression(None, Some("port.listen|tcp|10.0.0.5:"));
        assert!(entry.covers(
            "port.listen|tcp|10.0.0.5:9000",
            "port.listen.new",
            None,
            "2026-09-09T10:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:9000",
            "port.listen.new",
            None,
            "2026-09-09T10:00:00.000Z"
        ));

        entry.kind = Some("port.listen.new".into());
        assert!(
            !entry.covers(
                "port.listen|tcp|10.0.0.5:9000",
                "process.binary_deleted",
                None,
                "2026-09-09T10:00:00.000Z"
            ),
            "silencing 'the port appeared' must not silence 'its binary was deleted'"
        );
    }

    #[test]
    fn an_expired_suppression_stops_covering_without_anybody_editing_the_file() {
        let mut entry = suppression(Some("port.listen|tcp|0.0.0.0:8080"), None);
        entry.until = Some("2026-09-09T09:00:00.000Z".into());

        assert!(entry.covers(
            "port.listen|tcp|0.0.0.0:8080",
            "port.listen.new",
            None,
            "2026-09-09T08:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:8080",
            "port.listen.new",
            None,
            "2026-09-09T10:00:00.000Z"
        ));
    }

    const NOW: &str = "2026-10-30T10:00:00.000Z";

    fn tunnel() -> Suppression {
        Suppression {
            program: Some("netbird".into()),
            finding_key_match: Some("port.listen|udp|*".into()),
            reason: "vpn mesh changes its ports".into(),
            ..Suppression::default()
        }
    }

    #[test]
    fn an_entry_written_before_programs_and_patterns_still_reads_as_it_did() {
        let entry: Suppression = serde_yaml::from_str(
            "finding_key_prefix: \"port.listen|tcp|10.0.0.5:\"\nkind: port.listen.new\nreason: lab\n",
        )
        .expect("an old entry loads");

        assert!(entry.validate().is_ok());
        assert_eq!(entry.program, None);
        assert_eq!(entry.finding_key_match, None);
    }

    #[test]
    fn an_entry_naming_a_program_alone_is_a_statement_of_its_own() {
        let entry: Suppression =
            serde_yaml::from_str("program: netbird\nreason: vpn mesh changes its ports\n")
                .expect("a program alone loads");

        assert!(entry.validate().is_ok());
    }

    #[test]
    fn an_entry_that_names_nothing_lists_every_way_there_is_to_name_something() {
        let entry = Suppression {
            reason: "nothing to silence".into(),
            ..Suppression::default()
        };

        assert_eq!(
            entry.validate(),
            Err("a suppression needs finding_key, finding_key_prefix, finding_key_match, kind or program; as written it silences nothing".into())
        );
    }

    #[test]
    fn a_key_named_in_two_ways_is_refused_and_the_two_are_named() {
        let entry = Suppression {
            finding_key_prefix: Some("port.listen|udp|".into()),
            finding_key_match: Some("port.listen|udp|*".into()),
            reason: "two ways".into(),
            ..Suppression::default()
        };

        assert_eq!(
            entry.validate(),
            Err("a suppression names the key in two ways at once (finding_key_prefix, finding_key_match); keep one".into())
        );
    }

    #[test]
    fn a_pattern_of_stars_alone_is_refused_because_it_would_be_a_kind_in_disguise() {
        let program = Suppression {
            program: Some("*".into()),
            reason: "anything".into(),
            ..Suppression::default()
        };
        let key = Suppression {
            finding_key_match: Some("**".into()),
            reason: "anything".into(),
            ..Suppression::default()
        };

        assert_eq!(
            program.validate(),
            Err(
                "program \"*\" matches every program; name the program, or silence by kind instead"
                    .into()
            )
        );
        assert_eq!(
            key.validate(),
            Err("finding_key_match \"**\" matches every key; name the key, or silence by kind instead".into())
        );
    }

    #[test]
    fn a_program_that_changes_its_ports_is_silenced_by_one_entry_and_nothing_else_with_it() {
        let entry = tunnel();
        let netbird = Some("/usr/bin/netbird");

        assert!(entry.covers(
            "port.listen|udp|0.0.0.0:41234",
            "port.listen.new",
            netbird,
            NOW
        ));
        assert!(entry.covers(
            "port.listen|udp|0.0.0.0:39001",
            "port.listen.removed",
            netbird,
            NOW
        ));
        assert!(
            !entry.covers(
                "port.listen|tcp|0.0.0.0:443",
                "port.listen.new",
                netbird,
                NOW
            ),
            "the pattern names udp, and a tcp port of the same program is something else"
        );
        assert!(
            !entry.covers(
                "port.listen|udp|0.0.0.0:41234",
                "port.listen.new",
                Some("/usr/sbin/sshd"),
                NOW
            ),
            "another program on a port of the same shape is not the tunnel"
        );
    }

    #[test]
    fn a_program_named_without_a_slash_is_its_file_name_and_with_one_its_path() {
        let by_name = Suppression {
            program: Some("netbird".into()),
            reason: "vpn".into(),
            ..Suppression::default()
        };
        let by_path = Suppression {
            program: Some("/opt/*/netbird".into()),
            reason: "vpn".into(),
            ..Suppression::default()
        };
        let key = "port.listen|udp|0.0.0.0:41234";

        assert!(by_name.covers(key, "port.listen.new", Some("/usr/bin/netbird"), NOW));
        assert!(by_name.covers(
            key,
            "port.listen.new",
            Some("/opt/netbird/bin/netbird"),
            NOW
        ));
        assert!(!by_name.covers(key, "port.listen.new", Some("/usr/bin/netbird-ui"), NOW));
        assert!(by_path.covers(
            key,
            "port.listen.new",
            Some("/opt/netbird/bin/netbird"),
            NOW
        ));
        assert!(!by_path.covers(key, "port.listen.new", Some("/usr/bin/netbird"), NOW));
    }

    #[test]
    fn an_entry_naming_a_program_never_covers_a_finding_that_has_none() {
        let entry = Suppression {
            program: Some("netbird".into()),
            reason: "vpn".into(),
            ..Suppression::default()
        };

        assert!(!entry.covers("files|/etc/netbird/config.json", "file.changed", None, NOW));
    }

    #[test]
    fn a_kind_next_to_a_program_narrows_it_to_that_statement() {
        let entry = Suppression {
            program: Some("netbird".into()),
            kind: Some("port.listen.new".into()),
            reason: "vpn".into(),
            ..Suppression::default()
        };
        let netbird = Some("/usr/bin/netbird");

        assert!(entry.covers(
            "port.listen|tcp|0.0.0.0:443",
            "port.listen.new",
            netbird,
            NOW
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:443",
            "port.listen.removed",
            netbird,
            NOW
        ));
        assert!(!entry.covers(
            "exec|/usr/bin/netbird|root",
            "process.binary_deleted",
            netbird,
            NOW
        ));
    }

    #[test]
    fn a_program_and_a_pattern_are_described_the_way_the_status_shows_them() {
        let only_program = Suppression {
            program: Some("netbird".into()),
            kind: Some("port.listen.new".into()),
            reason: "vpn mesh".into(),
            ..Suppression::default()
        };
        let mut both = tunnel();
        both.reason = "vpn mesh".into();

        assert_eq!(
            both.describe(),
            "program netbird · port.listen|udp|* — vpn mesh"
        );
        assert_eq!(
            only_program.describe(),
            "program netbird (port.listen.new) — vpn mesh"
        );
        assert_eq!(
            suppression(None, Some("port.listen|tcp|10.0.0.5:")).describe(),
            "port.listen|tcp|10.0.0.5:* — expected on this host"
        );
    }

    #[test]
    fn the_same_key_with_and_without_a_program_are_two_statements() {
        let pattern = Suppression {
            finding_key_match: Some("port.listen|udp|*".into()),
            reason: "any".into(),
            ..Suppression::default()
        };

        assert!(tunnel().says_the_same_as(&tunnel()));
        assert!(!tunnel().says_the_same_as(&pattern));
        assert!(
            !pattern.says_the_same_as(&suppression(None, Some("port.listen|udp|*"))),
            "a pattern and a prefix spelled alike cover different things"
        );
    }
}
