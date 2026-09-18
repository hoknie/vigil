use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Suppression {
    pub finding_key: Option<String>,
    pub finding_key_prefix: Option<String>,
    pub kind: Option<String>,
    pub reason: String,
    pub until: Option<String>,
}

impl Suppression {
    pub fn validate(&self) -> Result<(), String> {
        match (&self.finding_key, &self.finding_key_prefix) {
            (None, None) if self.kind.is_none() => Err(
                "a suppression needs finding_key, finding_key_prefix or kind; as written it silences nothing"
                    .into(),
            ),
            (Some(_), Some(_)) => Err(
                "a suppression names finding_key and finding_key_prefix at once; keep one".into(),
            ),
            _ if self.reason.trim().is_empty() => Err("a suppression needs a reason".into()),
            _ => Ok(()),
        }
    }

    pub fn covers(&self, finding_key: &str, kind: &str, now: &str) -> bool {
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
        match (&self.finding_key, &self.finding_key_prefix) {
            (Some(exact), _) => exact == finding_key,
            (_, Some(prefix)) => finding_key.starts_with(prefix.as_str()),
            _ => self.kind.is_some(),
        }
    }

    pub fn says_the_same_as(&self, other: &Suppression) -> bool {
        self.finding_key == other.finding_key
            && self.finding_key_prefix == other.finding_key_prefix
            && self.kind == other.kind
    }

    pub fn describe(&self) -> String {
        let what = match (&self.finding_key, &self.finding_key_prefix) {
            (Some(exact), _) => exact.clone(),
            (_, Some(prefix)) => format!("{prefix}*"),
            _ => "anywhere".to_string(),
        };
        match &self.kind {
            Some(kind) => format!("{what} ({kind}) — {}", self.reason),
            None => format!("{what} — {}", self.reason),
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
            "2026-09-09T10:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:80",
            "port.listen.new",
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
            "2026-09-09T10:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:8081",
            "port.listen.new",
            "2026-09-09T10:00:00.000Z"
        ));
    }

    #[test]
    fn a_prefix_covers_a_range_and_a_kind_narrows_it() {
        let mut entry = suppression(None, Some("port.listen|tcp|10.0.0.5:"));
        assert!(entry.covers(
            "port.listen|tcp|10.0.0.5:9000",
            "port.listen.new",
            "2026-09-09T10:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:9000",
            "port.listen.new",
            "2026-09-09T10:00:00.000Z"
        ));

        entry.kind = Some("port.listen.new".into());
        assert!(
            !entry.covers(
                "port.listen|tcp|10.0.0.5:9000",
                "process.binary_deleted",
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
            "2026-09-09T08:00:00.000Z"
        ));
        assert!(!entry.covers(
            "port.listen|tcp|0.0.0.0:8080",
            "port.listen.new",
            "2026-09-09T10:00:00.000Z"
        ));
    }
}
