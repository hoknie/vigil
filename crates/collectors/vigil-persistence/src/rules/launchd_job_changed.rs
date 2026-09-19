use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build};
use crate::types::{Family, PersistenceView};
use vigil_rules::{Rule, RuleContext};

pub struct LaunchdJobChanged;

impl Rule for LaunchdJobChanged {
    fn name(&self) -> &'static str {
        "launchd_job_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Launchd) {
            return None;
        }
        let was = PersistenceView::new(key, before);

        if was.commands() == view.commands()
            && was.run_as() == view.run_as()
            && was.inserted_libraries() == view.inserted_libraries()
        {
            return None;
        }
        if was.readable() && !view.readable() {
            return None;
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::FileChanged,
                severity: match view.runs_from_writable_path()
                    || !view.inserted_libraries().is_empty()
                {
                    true => Severity::Critical,
                    false => Severity::High,
                },
                rule: self.name(),
                key,
                object: "launchd_job",
                title: format!(
                    "launchd {} {} now runs {} as {}",
                    view.domain(),
                    view.name(),
                    view.commands(),
                    view.run_as()
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    Evidence {
                        kind: "path".into(),
                        value: view.path().to_string(),
                    },
                    Evidence {
                        kind: "cmdline".into(),
                        value: format!("was: {} as {}", was.commands(), was.run_as()),
                    },
                ],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    const PATH: &str = "/Library/LaunchDaemons/com.example.updater.plist";

    fn apply(before: serde_json::Value, after: serde_json::Value) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-19T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        LaunchdJobChanged.apply(
            &Change::Changed {
                key: format!("launchd|{PATH}"),
                before,
                after,
            },
            &mut ctx,
        )
    }

    fn job(program: &str) -> serde_json::Value {
        fixture::launchd_job(PATH, program, "daemon", "system")
    }

    #[test]
    fn a_job_pointed_at_another_program_is_reported_with_both() {
        let finding =
            apply(job("/usr/local/bin/updater"), job("/Users/Shared/updater")).expect("fires");

        assert_eq!(finding.kind.as_str(), "file.changed");
        assert_eq!(finding.severity, Severity::Critical);
        assert!(
            finding
                .evidence
                .iter()
                .any(|said| said.value.contains("/usr/local/bin/updater")),
            "{:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_job_that_started_loading_a_library_into_its_program_is_the_same_event() {
        let mut after = job("/usr/local/bin/updater");
        after["inserted_libraries"] = serde_json::json!(["/usr/local/lib/hook.dylib"]);

        let finding = apply(job("/usr/local/bin/updater"), after).expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn a_schedule_that_moved_is_not_a_different_program() {
        let mut after = job("/usr/local/bin/updater");
        after["schedule"] = serde_json::json!("every 60 s");

        assert!(apply(job("/usr/local/bin/updater"), after).is_none());
    }

    #[test]
    fn a_job_the_agent_stopped_being_able_to_read_is_not_a_job_that_runs_nothing() {
        let mut after = job("/usr/local/bin/updater");
        after["readable"] = serde_json::json!(false);
        after["commands"] = serde_json::json!([]);

        assert!(apply(job("/usr/local/bin/updater"), after).is_none());
    }
}
