use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::resource_view::{Family, ResourceView};
use crate::RuleContext;

pub struct ResourceFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub finding_key: String,
    pub object: &'static str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

pub fn build(spec: ResourceFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let describing = spec.after.as_ref().or(spec.before.as_ref());
    let subject = match describing {
        Some(value) => {
            let view = ResourceView::new(spec.key, value);
            match view.family() {
                Some(Family::Filesystem) => json!({
                    "mount": view.mount(),
                    "device": view.device(),
                    "type": view.kind(),
                }),
                _ => json!({ "host": "this host" }),
            }
        }
        None => json!({ "key": spec.key }),
    };

    let now = ctx.now.clone();
    Finding {
        event_id: (ctx.mint_event_id)(),
        finding_key: spec.finding_key,
        kind: Kind::Known(spec.kind),
        severity: spec.severity,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: spec.title,
        subject: Subject {
            object: spec.object.into(),
            key: subject,
        },
        before: spec.before,
        after: spec.after,
        evidence: spec.evidence,
        redacted: Vec::new(),
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn room(view: &ResourceView<'_>) -> Evidence {
    Evidence {
        kind: "note".into(),
        value: format!(
            "{} is {} on {}, {} bytes in all; at least {}% of it is free and at least {} of its inodes",
            view.mount(),
            view.kind(),
            view.device(),
            view.total_bytes(),
            view.free_percent_step().unwrap_or(0),
            match view.free_inodes_percent_step() {
                Some(step) => format!("{step}%"),
                None => "an unknown share".to_string(),
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    #[test]
    fn a_finding_about_a_filesystem_names_the_mount_point_a_person_would_look_for() {
        let filesystem = fixture::filesystem("/var", Some(5), Some(85));
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };

        let finding = build(
            ResourceFinding {
                kind: KnownKind::ResourceDiskLow,
                severity: Severity::High,
                rule: "disk_low",
                key: "fs|/var",
                finding_key: "resource|disk|/var".into(),
                object: "filesystem",
                title: "A filesystem is filling up".into(),
                before: None,
                after: Some(filesystem.clone()),
                evidence: vec![room(&ResourceView::new("fs|/var", &filesystem))],
            },
            &mut ctx,
        );

        assert_eq!(finding.subject.key["mount"], "/var");
        assert_eq!(finding.subject.object, "filesystem");
        assert_eq!(finding.finding_key, "resource|disk|/var");
        assert!(finding.evidence[0].value.contains("ext4"));
    }
}
