use std::path::{Path, PathBuf};

use vigil_config::Block;

use crate::Config;
use vigil_collect::{Health, Limit, Standing};
use vigil_model::{Kind, KnownKind};

use crate::boot::greeting::{collectors, health_findings, health_lines};

fn block(name: &str, file: &str, enabled: bool) -> Block {
    Block {
        name: name.to_string(),
        file: PathBuf::from(file),
        enabled,
        schedule: None,
        settings: yaml_serde::Mapping::new(),
    }
}

fn apart(at: &Path, blocks: Vec<Block>) -> Config {
    let mut config = Config {
        collectors: Some(
            blocks
                .iter()
                .filter(|block| block.enabled)
                .map(|block| block.name.clone())
                .collect(),
        ),
        ..Config::default()
    };
    config.apart.collectors_at = Some(at.to_path_buf());
    config.apart.collectors = blocks;
    config
}

#[test]
fn where_the_collectors_are_read_from_and_which_blocks_are_off_is_said_at_start_up() {
    let config = apart(
        Path::new("/etc/vigil/collectors"),
        vec![
            block("users", "/etc/vigil/collectors/users.yaml", true),
            block("files", "/etc/vigil/collectors/files.yaml", false),
        ],
    );

    let said = collectors(
        "/etc/vigil/vigil.yaml",
        &config,
        &["users"],
        &["files".to_string(), "launches".to_string()],
    )
    .join("\n");

    assert!(said.contains("read from /etc/vigil/collectors"), "{said}");
    assert!(
        said.contains(
            "collector files: off — `enabled: false` in /etc/vigil/collectors/files.yaml"
        ),
        "an operator reading the start-up learns which line in which file switched it off: {said}"
    );
    assert!(
        said.contains("collector launches: off — no block for it in /etc/vigil/collectors"),
        "{said}"
    );
    assert!(!said.contains("`collectors:`"), "{said}");
}

#[test]
fn a_directory_that_is_not_there_and_one_with_nothing_in_it_are_each_said_in_words_of_their_own() {
    let directory = std::env::temp_dir().join(format!("vigild-greeting-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a directory");

    let absent = collectors(
        "/etc/vigil/vigil.yaml",
        &apart(&directory.join("collectors"), Vec::new()),
        &[],
        &[],
    )
    .join("\n");
    let empty = collectors(
        "/etc/vigil/vigil.yaml",
        &apart(&directory, Vec::new()),
        &[],
        &[],
    )
    .join("\n");
    let all_off = collectors(
        "/etc/vigil/vigil.yaml",
        &apart(&directory, vec![block("users", "users.yaml", false)]),
        &[],
        &["users".to_string()],
    )
    .join("\n");

    assert!(
        absent.contains("NONE") && absent.contains("is not there"),
        "{absent}"
    );
    assert!(
        empty.contains("NONE") && empty.contains("holds no block"),
        "{empty}"
    );
    assert!(
        all_off.contains("NONE") && all_off.contains("enabled: false"),
        "{all_off}"
    );
    assert_ne!(absent, empty);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_file_of_the_former_layout_is_still_told_about_in_the_words_it_was_written_in() {
    let config = Config {
        collectors: Some(Vec::new()),
        ..Config::default()
    };

    let said = collectors("/etc/vigil/vigil.yaml", &config, &[], &[]).join("\n");

    assert!(
        said.contains("`collectors:` in /etc/vigil/vigil.yaml is empty"),
        "{said}"
    );
    assert!(!said.contains("read from"), "{said}");
}

fn limit(said: &str) -> Option<Limit> {
    Limit::of(vec![said.to_string()])
}

#[test]
fn a_limited_collector_says_so_at_start_and_raises_no_degraded() {
    let standings = [
        (
            "processes",
            Standing::limited(
                Health::Ok,
                limit("the command lines of the processes of every other account are not read"),
            ),
        ),
        ("resources", Standing::of(Health::Ok)),
    ];

    assert_eq!(
        health_lines(&standings),
        [
            "  collector processes: limited — the command lines of the processes of every other \
             account are not read",
            "  collector resources: ok",
        ]
    );
    let findings = health_findings(&standings, "ann (uid 501)");
    let kinds: Vec<&Kind> = findings.iter().map(|finding| &finding.kind).collect();
    assert_eq!(kinds, [&Kind::Known(KnownKind::AgentCollectorLimited)]);
    assert_eq!(findings[0].finding_key, "agent.collector.limited|processes");
    assert_eq!(findings[0].severity, vigil_model::Severity::Low);
}

#[test]
fn a_limited_and_degraded_collector_says_both_at_start() {
    let standings = [(
        "users",
        Standing::limited(
            Health::Degraded("utmpx is not there: who is logged in will be missing".into()),
            limit("/private/etc/sudoers is not readable"),
        ),
    )];

    assert_eq!(
        health_lines(&standings),
        [
            "  collector users: degraded — utmpx is not there: who is logged in will be missing",
            "  collector users: limited — /private/etc/sudoers is not readable",
        ]
    );
    let keys: Vec<(String, String)> = health_findings(&standings, "ann (uid 501)")
        .into_iter()
        .map(|finding| (finding.finding_key, finding.kind.to_string()))
        .collect();
    assert_eq!(
        keys,
        [
            (
                "agent.collector|users".to_string(),
                "agent.collector.degraded".to_string()
            ),
            (
                "agent.collector.limited|users".to_string(),
                "agent.collector.limited".to_string()
            ),
        ]
    );
}
