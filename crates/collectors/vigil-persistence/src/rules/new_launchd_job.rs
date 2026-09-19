use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build, origin_evidence};
use crate::types::{Family, PersistenceView};
use vigil_rules::{Rule, RuleContext};

const VENDOR: &str = "vendor";

pub struct NewLaunchdJob;

impl Rule for NewLaunchdJob {
    fn name(&self) -> &'static str {
        "new_launchd_job"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Launchd) {
            return None;
        }

        let inserted = view.inserted_libraries();
        let severity = match (
            view.runs_from_writable_path() || !inserted.is_empty(),
            view.scope() == VENDOR,
        ) {
            (true, _) => Severity::Critical,
            (false, true) => Severity::Low,
            (false, false) => Severity::Medium,
        };

        let mut evidence = origin_evidence(&view);
        if !inserted.is_empty() {
            evidence.push(Evidence {
                kind: "note".into(),
                value: format!(
                    "DYLD_INSERT_LIBRARIES loads {} into the program before its own code runs",
                    inserted.join(", ")
                ),
            });
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceLaunchdNew,
                severity,
                rule: self.name(),
                key,
                object: "launchd_job",
                title: match view.commands().is_empty() {
                    true => format!("New launchd {} {}", view.domain(), view.name()),
                    false => format!(
                        "New launchd {} {} running {} as {}",
                        view.domain(),
                        view.name(),
                        view.commands(),
                        view.run_as()
                    ),
                },
                before: None,
                after: Some(after.clone()),
                evidence,
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-19T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        NewLaunchdJob.apply(change, &mut ctx)
    }

    fn added(path: &str, after: serde_json::Value) -> Change {
        Change::Added {
            key: format!("launchd|{path}"),
            after,
        }
    }

    #[test]
    fn a_new_daemon_is_reported_by_its_label_with_what_it_runs_and_as_whom() {
        let path = "/Library/LaunchDaemons/com.example.updater.plist";
        let finding = apply(&added(
            path,
            fixture::launchd_job(path, "/usr/local/bin/updater", "daemon", "system"),
        ))
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "persistence.launchd.new");
        assert_eq!(finding.severity, Severity::Medium);
        assert_eq!(finding.finding_key, format!("persistence|launchd|{path}"));
        assert_eq!(finding.subject.object, "launchd_job");
        assert!(
            finding
                .title
                .contains("daemon com.example.updater running /usr/local/bin/updater as root"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn a_job_that_starts_a_program_anybody_could_have_written_outranks_every_other() {
        let path = "/Users/alice/Library/LaunchAgents/com.apple.update.plist";
        let finding = apply(&added(
            path,
            fixture::launchd_job(path, "/private/tmp/.x/agent", "agent", "person"),
        ))
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn a_job_that_loads_a_library_into_its_program_is_critical_whatever_the_program_is() {
        let path = "/Library/LaunchAgents/com.example.helper.plist";
        let mut job = fixture::launchd_job(path, "/usr/bin/true", "agent", "system");
        job["inserted_libraries"] = serde_json::json!(["/usr/local/lib/hook.dylib"]);

        let finding = apply(&added(path, job)).expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert!(
            finding
                .evidence
                .iter()
                .any(|said| said.value.contains("DYLD_INSERT_LIBRARIES")),
            "{:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_job_of_the_system_volume_is_low_because_only_an_update_of_macos_writes_there() {
        let path = "/System/Library/LaunchDaemons/com.apple.newthing.plist";
        let finding = apply(&added(
            path,
            fixture::launchd_job(path, "/usr/libexec/newthing", "daemon", "vendor"),
        ))
        .expect("fires");

        assert_eq!(
            finding.severity,
            Severity::Low,
            "the system volume is sealed, and a job that appears on it arrived with an update"
        );
    }

    #[test]
    fn a_systemd_unit_is_somebody_elses_rule() {
        let change = Change::Added {
            key: "unit|nginx.service".into(),
            after: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
        };

        assert!(apply(&change).is_none());
    }
}
