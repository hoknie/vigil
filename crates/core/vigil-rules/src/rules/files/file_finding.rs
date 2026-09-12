use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::file_view::FileView;
use crate::RuleContext;

pub struct FileFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub object: &'static str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

pub fn build(spec: FileFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let describing = spec.after.as_ref().or(spec.before.as_ref());
    let path = describing
        .map(|value| FileView::new(spec.key, value).path().to_string())
        .unwrap_or_else(|| spec.key.to_string());

    let now = ctx.now.clone();
    Finding {
        event_id: (ctx.mint_event_id)(),
        finding_key: finding_key(spec.key),
        kind: Kind::Known(spec.kind),
        severity: spec.severity,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: spec.title,
        subject: Subject {
            object: spec.object.into(),
            key: json!({ "path": path }),
        },
        before: spec.before,
        after: spec.after,
        evidence: spec.evidence,
        redacted: Vec::new(),
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn finding_key(key: &str) -> String {
    key.to_string()
}

pub fn standing(view: &FileView<'_>) -> Evidence {
    Evidence {
        kind: "note".into(),
        value: match view.present() {
            true => format!(
                "{} is {} bytes, mode {}, owned by {}:{}",
                view.path(),
                view.size(),
                view.mode().unwrap_or("a mode this agent could not read"),
                view.owner()
                    .0
                    .map(|uid| uid.to_string())
                    .unwrap_or_default(),
                view.owner()
                    .1
                    .map(|gid| gid.to_string())
                    .unwrap_or_default(),
            ),
            false => format!("{} is not on this host", view.path()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_of_a_finding_about_a_file_is_the_path_a_person_would_write_in_a_suppression() {
        assert_eq!(
            finding_key("file|/etc/ssh/sshd_config"),
            "file|/etc/ssh/sshd_config",
            "the key of the row already is the object: a file is the same thing to the \
             collector, to the rule and to the reader, and rewriting it here would only make \
             the two disagree"
        );
        assert_eq!(finding_key("directory|/usr/bin"), "directory|/usr/bin");
    }
}
