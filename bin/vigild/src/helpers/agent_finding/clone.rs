use serde_json::json;
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::helpers::{rfc3339, uuid7};
use crate::identity::Moved;

pub fn clone_suspected(moved: &Moved) -> Finding {
    let now = rfc3339::now();
    let mut evidence = vec![
        Evidence {
            kind: "note".into(),
            value: format!(
                "{}: an installation copied with the image it was installed on reads another machine at its first start",
                moved.change.said()
            ),
        },
        Evidence {
            kind: "host_id".into(),
            value: format!("{} before, {} now", moved.previous_host_id, moved.host_id),
        },
    ];
    evidence.push(Evidence {
        kind: "install_id".into(),
        value: match &moved.install_id {
            Some(minted) => format!(
                "{} before, {minted} now: this host reports under the new one from this start on",
                moved.previous_install_id
            ),
            None => format!(
                "{} kept: a new one could not be written, and the next start tries again",
                moved.previous_install_id
            ),
        },
    });

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.clone|{}", moved.previous_install_id),
        kind: Kind::Known(KnownKind::AgentCloneSuspected),
        severity: Severity::High,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: "This installation now runs on another machine: the image was probably cloned"
            .into(),
        subject: Subject {
            object: "installation".into(),
            key: json!({ "install_id": moved.previous_install_id }),
        },
        before: None,
        after: None,
        evidence,
        redacted: Vec::new(),
        rule: Some("installation_moved".into()),
        labels: Default::default(),
    }
}
