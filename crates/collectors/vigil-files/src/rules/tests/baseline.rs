use vigil_model::Snapshot;
use vigil_rules::RuleContext;

use crate::fixture;
use crate::rules::file_rules;

fn judged(baseline: &Snapshot) -> Vec<String> {
    let mut mint = || "event-1".to_string();
    let mut ctx = RuleContext {
        now: "2026-09-11T12:04:02.104Z".into(),
        mint_event_id: &mut mint,
    };
    file_rules()
        .judge_baseline(baseline, &mut ctx)
        .into_iter()
        .map(|finding| format!("{} {}", finding.kind, finding.finding_key))
        .collect()
}

#[test]
fn a_directory_of_the_path_open_to_everybody_before_the_agent_came_is_said_on_its_first_reading() {
    let mut baseline = Snapshot::new("files", "2026-09-11T12:04:02.104Z");
    baseline.items.insert(
        "directory|/usr/local/bin".into(),
        fixture::watched_directory("/usr/local/bin", "0777"),
    );
    baseline.items.insert(
        "directory|/usr/bin".into(),
        fixture::watched_directory("/usr/bin", "0755"),
    );
    baseline.items.insert(
        "file|/usr/bin/at".into(),
        fixture::watched_file("/usr/bin/at", "4755", "a9"),
    );

    assert_eq!(
        judged(&baseline),
        vec!["file.path_writable_by_all directory|/usr/local/bin".to_string()],
        "the open directory is dangerous whenever it was opened; a setuid program and a \
         closed directory on the first reading are the host as it is, and say nothing"
    );
}

#[test]
fn the_only_file_rule_that_speaks_on_the_baseline_is_the_open_directory_of_the_path() {
    assert_eq!(file_rules().baseline_judges(), vec!["path_writable_by_all"]);
}
