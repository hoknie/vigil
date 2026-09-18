use super::load;

#[test]
fn an_empty_file_is_a_working_configuration() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("empty.yaml");
    std::fs::write(&path, "{}\n").expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.state_dir, "/var/lib/vigil");
    assert!(
        config.reporters.is_empty(),
        "no receiver configured is a supported configuration"
    );
}

#[test]
fn a_suppression_that_covers_nothing_is_refused_at_start_up() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("empty-suppression.yaml");
    std::fs::write(&path, "suppressions:\n  - reason: the staging api\n").expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("suppression #1"), "{error}");
    assert!(error.cause.contains("silences nothing"), "{error}");
}

#[test]
fn a_suppression_that_names_an_object_and_a_reason_is_accepted() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("suppression.yaml");
    std::fs::write(
        &path,
        "suppressions:\n  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"\n    reason: the staging api, expected\n",
    )
    .expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.suppressions.len(), 1);
    assert_eq!(config.suppressions[0].reason, "the staging api, expected");
}

#[test]
fn a_misspelled_syslog_facility_is_refused_at_start_up_rather_than_sent_to_nowhere() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("facility.yaml");
    std::fs::write(
        &path,
        "reporters:\n  - kind: syslog\n    facility: lokal0\n",
    )
    .expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("reporter #1"), "{error}");
    assert!(error.cause.contains("lokal0"), "{error}");
    assert!(
        error.cause.contains("local0"),
        "the message says what was allowed: {error}"
    );
}

#[test]
fn a_facility_every_syslog_daemon_knows_is_accepted() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("facility-ok.yaml");
    std::fs::write(
        &path,
        "reporters:\n  - kind: syslog\n    facility: local4\n",
    )
    .expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.reporters.len(), 1);
}

#[test]
fn the_file_this_product_ships_is_a_file_this_product_reads() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../config/vigil.example.yaml"
    );

    let config = load(path).expect("the shipped example must load");

    assert_eq!(
        config.collectors,
        Some(
            crate::modules::names()
                .iter()
                .map(|name| name.to_string())
                .collect()
        ),
        "the shipped example names every collector this product has, in the product's order"
    );
}

#[test]
fn a_configuration_with_no_collectors_key_watches_everything() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("no-collectors.yaml");
    std::fs::write(&path, "interval_seconds: 30\n").expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.collectors, None);
}

#[test]
fn an_empty_collector_list_is_a_deliberate_nothing_and_not_the_same_as_no_key() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("no-collector.yaml");
    std::fs::write(&path, "collectors: []\n").expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.collectors, Some(Vec::new()));
}

#[test]
fn a_misspelled_collector_is_refused_by_name_and_told_which_ones_exist() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("collector-typo.yaml");
    std::fs::write(&path, "collectors:\n  - ports\n  - proccesses\n").expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("proccesses"), "{error}");
    assert!(error.cause.contains("processes"), "{error}");
    assert!(error.cause.contains("collector #2"), "{error}");
}

#[test]
fn a_collector_named_twice_is_refused_rather_than_watched_twice() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("collector-twice.yaml");
    std::fs::write(&path, "collectors: [ports, users, ports]\n").expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("named twice"), "{error}");
}

#[test]
fn a_value_a_module_refuses_stops_the_daemon_at_the_door_and_not_at_the_first_reading() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("module-value.yaml");
    std::fs::write(&path, "resources:\n  disk_free_percent: 101\n").expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("resources"), "{error}");
    assert!(error.cause.contains("disk_free_percent"), "{error}");
}

#[test]
fn a_misspelled_key_inside_a_module_block_is_refused_by_the_module_that_owns_it() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("module-typo.yaml");
    std::fs::write(&path, "files:\n  celing_bytes: 2048\n").expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be ignored");

    assert!(error.cause.contains("celing_bytes"), "{error}");
    assert!(
        error.cause.contains("files"),
        "the message says whose key it was: {error}"
    );
}

#[test]
fn a_module_block_the_file_names_reaches_the_module_and_not_the_daemons_own_fields() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("module-block.yaml");
    std::fs::write(
        &path,
        "retention_days: 5\nlaunches:\n  record_arguments: true\n",
    )
    .expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.retention_days, 5);
    assert_eq!(
        config.of_the_module("launches"),
        serde_json::json!({"record_arguments": true}),
        "the daemon carries the block whole and hands it over: what is in it is between \
         the module and the operator"
    );
}

#[test]
fn a_misspelled_field_is_refused_by_name_instead_of_ignored() {
    let dir = std::env::temp_dir().join("vigil-config-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("typo.yaml");
    std::fs::write(&path, "retention_dayz: 5\n").expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be ignored");

    assert!(error.cause.contains("retention_dayz"), "{error}");
    assert!(error.to_string().contains("typo.yaml"), "{error}");
}

fn bench(named: &str) -> std::path::PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-config-apart-{named}-{}-{}",
        std::process::id(),
        NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("temp dir");
    directory
}

#[test]
fn suppressions_kept_in_a_directory_of_their_own_are_in_force_beside_the_configurations_own() {
    let directory = bench("suppressions");
    let path = directory.join("vigil.yaml");
    std::fs::write(
        &path,
        "suppressions:\n  - finding_key: \"user|group|docker\"\n    reason: ours\nsuppressions_path: suppressions\n",
    )
    .expect("write");
    std::fs::create_dir_all(directory.join("suppressions")).expect("a directory");
    std::fs::write(
        directory.join("suppressions").join("deploy.yaml"),
        "suppressions:\n  - finding_key_prefix: \"port.listen|tcp|10.0.0.5:\"\n    reason: staging\n",
    )
    .expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.suppressions.len(), 1);
    assert_eq!(config.every_suppression().len(), 2);
    assert_eq!(
        config.apart.suppressions_at,
        Some(directory.join("suppressions")),
        "a relative path is read beside the configuration, not beside wherever the daemon was started"
    );
}

#[test]
fn a_bad_entry_in_a_file_of_suppressions_stops_the_daemon_at_the_door_and_names_the_file() {
    let directory = bench("bad-suppression");
    let path = directory.join("vigil.yaml");
    std::fs::write(&path, "suppressions_path: suppressions\n").expect("write");
    std::fs::create_dir_all(directory.join("suppressions")).expect("a directory");
    std::fs::write(
        directory.join("suppressions").join("console.yaml"),
        "suppressions:\n  - reason: covers nothing\n",
    )
    .expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("console.yaml"), "{error}");
    assert!(error.cause.contains("suppression #1"), "{error}");
}

#[test]
fn a_directory_of_suppressions_nobody_made_yet_is_an_empty_one_and_the_daemon_starts() {
    let directory = bench("unmade");
    let path = directory.join("vigil.yaml");
    std::fs::write(
        &path,
        "suppressions_path: suppressions\nreporters_path: reporters\n",
    )
    .expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert!(config.every_suppression().is_empty());
    assert!(config.reporters.is_empty());
}

#[test]
fn a_path_key_written_as_nothing_is_refused_rather_than_read_as_the_directory_of_the_file() {
    let directory = bench("empty-path");
    let path = directory.join("vigil.yaml");
    std::fs::write(&path, "reporters_path: \"\"\n").expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("reporters_path is empty"), "{error}");
}

#[test]
fn reporters_kept_in_a_file_of_their_own_are_added_to_the_ones_the_configuration_names() {
    let directory = bench("reporters");
    let path = directory.join("vigil.yaml");
    std::fs::write(
        &path,
        "reporters:\n  - kind: syslog\n    facility: local4\nreporters_path: /REPLACED\n".replace(
            "/REPLACED",
            &directory.join("reporters").display().to_string(),
        ),
    )
    .expect("write");
    std::fs::create_dir_all(directory.join("reporters")).expect("a directory");
    std::fs::write(
        directory.join("reporters").join("journal.yaml"),
        "reporters:\n  - kind: ndjson\n    path: /var/log/vigil/findings.ndjson\n",
    )
    .expect("write");

    let config = load(path.to_str().expect("utf-8")).expect("parses");

    assert_eq!(config.reporters.len(), 2);
    assert!(matches!(
        config.reporters[0],
        crate::Receiver::Syslog { .. }
    ));
    assert!(matches!(
        config.reporters[1],
        crate::Receiver::Ndjson { .. }
    ));
    assert_eq!(config.apart.reporters.len(), 1);
}

#[test]
fn a_file_of_reporters_is_held_to_the_rules_the_configuration_is_and_names_itself() {
    let directory = bench("bad-reporter");
    let path = directory.join("vigil.yaml");
    std::fs::write(&path, "reporters_path: reporters\n").expect("write");
    std::fs::create_dir_all(directory.join("reporters")).expect("a directory");
    std::fs::write(
        directory.join("reporters").join("syslog.yaml"),
        "reporters:\n  - kind: syslog\n    facility: lokal0\n",
    )
    .expect("write");
    std::fs::write(
        directory.join("reporters").join("typo.yml"),
        "reporter:\n  - kind: ndjson\n    path: /tmp/x\n",
    )
    .expect("write");

    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");

    assert!(error.cause.contains("syslog.yaml"), "{error}");
    assert!(error.cause.contains("reporter #1"), "{error}");

    std::fs::remove_file(directory.join("reporters").join("syslog.yaml")).expect("removes");
    let error = load(path.to_str().expect("utf-8")).expect_err("must not be accepted");
    assert!(
        error.cause.contains("typo.yml") && error.cause.contains("reporter"),
        "a misspelled key in a file of reporters is a receiver somebody believes is sent to: {error}"
    );
}
