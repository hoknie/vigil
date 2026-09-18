use std::path::{Path, PathBuf};

use serde_json::json;

use super::bench::bench;
use crate::config::collectors::VERBS;
use crate::config::{Config, ConfigError, load};

const POINTING: &str = "collectors_path: collectors\n";

fn written(directory: &Path, file: &str, text: &str) {
    let path = directory.join(file);
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
    std::fs::write(path, text).expect("write");
}

fn apart(named: &str, files: &[(&str, &str)]) -> PathBuf {
    let directory = bench(named);
    written(&directory, "vigil.yaml", POINTING);
    for (file, text) in files {
        written(&directory, &format!("collectors/{file}"), text);
    }
    directory
}

fn loaded(directory: &Path) -> Config {
    load(directory.join("vigil.yaml").to_str().expect("utf-8")).expect("loads")
}

fn refused(directory: &Path) -> ConfigError {
    load(directory.join("vigil.yaml").to_str().expect("utf-8")).expect_err("must not be accepted")
}

fn on(config: &Config) -> Vec<&str> {
    config
        .collectors
        .as_ref()
        .expect("the blocks name what runs")
        .iter()
        .map(String::as_str)
        .collect()
}

#[test]
fn a_block_in_the_directory_switches_its_collector_on_and_brings_its_period_and_settings() {
    let directory = apart(
        "blocks",
        &[
            ("users.yaml", "users:\n  schedule: 5m\n"),
            (
                "resources.yaml",
                "resources:\n  schedule: 90\n  disk_free_percent: 20\n",
            ),
        ],
    );

    let config = loaded(&directory);

    assert_eq!(on(&config), ["users", "resources"]);
    assert_eq!(config.every_seconds("users"), 300);
    assert_eq!(config.every_seconds("resources"), 90);
    assert_eq!(
        config.of_the_module("resources"),
        json!({"disk_free_percent": 20}),
        "the module is handed its settings without the period, which is the daemon's"
    );
    assert_eq!(
        config.apart.collectors_at,
        Some(directory.join("collectors"))
    );
}

#[test]
fn a_block_that_says_enabled_false_is_off_and_its_period_is_not_a_period_of_a_collector_that_runs()
{
    let directory = apart(
        "off",
        &[
            ("users.yaml", "users:\n"),
            (
                "processes.yaml",
                "processes:\n  enabled: false\n  schedule: 30\n",
            ),
        ],
    );

    let config = loaded(&directory);

    assert_eq!(on(&config), ["users"]);
    assert!(
        !config.schedule.contains_key("processes"),
        "a period for a collector that does not run is refused by the daemon everywhere else: {:?}",
        config.schedule
    );
    assert!(
        config
            .apart
            .block("processes")
            .is_some_and(|block| !block.enabled),
        "the block is kept, so the start-up can say which file switched it off"
    );
}

#[test]
fn the_order_collectors_run_in_is_the_products_and_not_the_order_of_the_files() {
    let directory = apart(
        "order",
        &[
            ("a.yaml", "launches:\n"),
            ("b.yaml", "files:\n---\nusers:\n"),
        ],
    );

    let config = loaded(&directory);

    let expected: Vec<&str> = crate::modules::names()
        .into_iter()
        .filter(|name| ["launches", "files", "users"].contains(name))
        .collect();
    assert_eq!(on(&config), expected);
}

#[test]
fn a_collector_this_build_does_not_have_is_refused_with_its_file_and_the_ones_it_does() {
    let directory = apart("unknown", &[("typo.yaml", "proccesses:\n  schedule: 30\n")]);

    let error = refused(&directory);

    assert!(error.cause.contains("typo.yaml"), "{error}");
    assert!(error.cause.contains("proccesses"), "{error}");
    assert!(error.cause.contains("processes"), "{error}");
}

#[test]
fn each_console_verb_is_read_from_the_block_of_the_collector_it_acts_on() {
    let directory = apart(
        "verbs",
        &[
            (
                "processes.yaml",
                "processes:\n  killing:\n    from_the_console: true\n",
            ),
            (
                "users.yaml",
                "users:\n  accounts:\n    from_the_console: true\n",
            ),
            (
                "persistence.yaml",
                "persistence:\n  units:\n    from_the_console: true\n",
            ),
        ],
    );

    let config = loaded(&directory);

    assert!(config.killing.from_the_console);
    assert!(config.accounts.from_the_console);
    assert!(config.units.from_the_console);
    assert_eq!(
        config.of_the_module("users"),
        serde_json::Value::Null,
        "a verb is the daemon's and not a setting the module is handed"
    );
}

#[test]
fn a_verb_the_blocks_leave_out_is_off() {
    let directory = apart(
        "verbs-off",
        &[
            ("processes.yaml", "processes:\n"),
            ("users.yaml", "users:\n"),
        ],
    );

    let config = loaded(&directory);

    assert!(!config.killing.from_the_console);
    assert!(!config.accounts.from_the_console);
    assert!(!config.units.from_the_console);
}

#[test]
fn a_verb_written_in_the_block_of_another_collector_is_refused_by_name() {
    let directory = apart(
        "verb-elsewhere",
        &[(
            "users.yaml",
            "users:\n  killing:\n    from_the_console: true\n",
        )],
    );

    let error = refused(&directory);

    assert!(error.cause.contains("users.yaml"), "{error}");
    assert!(error.cause.contains("killing"), "{error}");
    assert!(
        error.cause.contains("processes"),
        "the message says where the word belongs: {error}"
    );
}

#[test]
fn a_verb_is_one_switch_and_a_misspelling_of_it_is_refused_rather_than_read_as_off() {
    for text in [
        "processes:\n  killing:\n    from_the_consle: true\n",
        "processes:\n  killing:\n    from_the_console: please\n",
    ] {
        let directory = apart("verb-typo", &[("processes.yaml", text)]);

        let error = refused(&directory);

        assert!(error.cause.contains("killing"), "{text:?}: {error}");
        assert!(error.cause.contains("processes.yaml"), "{text:?}: {error}");
    }
}

#[test]
fn a_setting_for_a_collector_that_takes_none_is_refused_rather_than_ignored() {
    let directory = apart(
        "no-settings",
        &[("users.yaml", "users:\n  record_arguments: true\n")],
    );

    let error = refused(&directory);

    assert!(error.cause.contains("users.yaml"), "{error}");
    assert!(error.cause.contains("record_arguments"), "{error}");
    assert!(error.cause.contains("takes no settings"), "{error}");
}

#[test]
fn a_setting_its_module_does_not_know_is_refused_and_the_file_it_is_in_is_named() {
    let directory = apart(
        "unknown-setting",
        &[("resources.yaml", "resources:\n  disk_free_prcent: 5\n")],
    );

    let error = refused(&directory);

    assert!(error.cause.contains("resources.yaml"), "{error}");
    assert!(error.cause.contains("disk_free_prcent"), "{error}");
}

#[test]
fn with_collectors_path_set_every_key_that_moved_is_refused_in_vigil_yaml_and_told_where_it_lives()
{
    for (key, text) in [
        ("collectors", "collectors: [users]\n"),
        ("schedule", "schedule:\n  users: 60\n"),
        ("interval_seconds", "interval_seconds: 30\n"),
        ("killing", "killing:\n  from_the_console: false\n"),
        ("accounts", "accounts:\n  from_the_console: false\n"),
        ("units", "units:\n  from_the_console: false\n"),
        ("resources", "resources:\n  disk_free_percent: 20\n"),
        ("users", "users: {}\n"),
        ("containers", "containers:\n  engines: [docker]\n"),
    ] {
        let directory = bench("moved");
        written(&directory, "vigil.yaml", &format!("{POINTING}{text}"));

        let error = refused(&directory);

        assert!(
            error.cause.starts_with(key) && error.cause.contains("collectors_path"),
            "two places for one setting is a place where an edit silently does nothing: {error}"
        );
    }
}

#[test]
fn the_engines_block_of_the_former_layout_is_sent_to_the_block_now_named_after_them() {
    let directory = bench("moved-engines");
    written(
        &directory,
        "vigil.yaml",
        &format!("{POINTING}containers:\n  engines: [docker]\n"),
    );

    let error = refused(&directory);

    assert!(
        error.cause.contains("block of containers-engines"),
        "the containers block holds the /proc reading now, and settings moved there would be \
         refused a second time: {error}"
    );
}

#[test]
fn a_directory_that_is_not_there_watches_nothing_and_the_daemon_still_starts() {
    let directory = bench("absent");
    written(&directory, "vigil.yaml", POINTING);

    let config = loaded(&directory);

    assert!(on(&config).is_empty());
    assert!(config.apart.collectors.is_empty());
    assert_eq!(
        config.apart.collectors_at,
        Some(directory.join("collectors"))
    );
}

#[test]
fn a_directory_with_no_block_in_it_watches_nothing_either() {
    let directory = apart("empty", &[("notes.yaml", "# nothing yet\n")]);

    let config = loaded(&directory);

    assert!(on(&config).is_empty());
}

#[test]
fn one_file_may_be_named_instead_of_a_directory_and_hold_every_block() {
    let directory = bench("one-file");
    written(
        &directory,
        "vigil.yaml",
        "collectors_path: collectors.yaml\n",
    );
    written(
        &directory,
        "collectors.yaml",
        "users:\n  schedule: 60\nlaunches:\n  record_arguments: true\n",
    );

    let config = loaded(&directory);

    assert_eq!(on(&config), ["users", "launches"]);
    assert_eq!(
        config.of_the_module("launches"),
        json!({"record_arguments": true})
    );
}

#[test]
fn the_block_of_the_container_engines_is_named_after_its_module() {
    let directory = apart(
        "engines",
        &[(
            "containers.yaml",
            "containers:\n  schedule: 60\n---\ncontainers-engines:\n  engines: [docker]\n",
        )],
    );

    let config = loaded(&directory);

    assert_eq!(on(&config), ["containers", "containers-engines"]);
    assert_eq!(
        config.of_the_module("containers-engines"),
        json!({"engines": ["docker"]})
    );
}

#[test]
fn every_verb_is_written_in_the_block_of_a_collector_this_build_has() {
    for (verb, owner) in VERBS {
        assert!(
            crate::modules::is_known(owner),
            "{verb} is read from the block of {owner}, and this build has no {owner}: the \
             switch could never be turned on"
        );
    }
}

#[test]
fn the_block_of_settings_of_every_module_is_named_after_the_module() {
    for module in crate::modules::modules() {
        if let Some(key) = module.settings_key() {
            assert_eq!(
                key,
                module.name(),
                "a block called one thing in `collectors:` and another in the settings is a \
                 block an operator writes in the wrong place"
            );
        }
    }
}

#[test]
fn a_file_of_the_former_layout_that_names_ports_is_read_as_the_collector_now_called_network() {
    let directory = bench("former-ports");
    written(
        &directory,
        "vigil.yaml",
        "collectors: [ports, users]\nschedule:\n  ports: 45\n",
    );

    let config = loaded(&directory);

    assert_eq!(
        on(&config),
        ["network", "users"],
        "an upgraded host keeps watching its sockets with the file it already had"
    );
    assert_eq!(config.every_seconds("network"), 45);
}

#[test]
fn a_file_of_the_former_layout_that_names_the_socket_collector_by_both_names_is_refused() {
    let directory = bench("former-both");
    written(&directory, "vigil.yaml", "collectors: [ports, network]\n");
    assert!(refused(&directory).cause.contains("named twice"));

    written(
        &directory,
        "vigil.yaml",
        "schedule:\n  ports: 30\n  network: 60\n",
    );
    let error = refused(&directory);
    assert!(
        error.cause.contains("ports") && error.cause.contains("network"),
        "{error}"
    );
}

#[test]
fn a_top_level_containers_block_of_the_former_layout_still_configures_the_engines() {
    let directory = bench("former-engines");
    written(
        &directory,
        "vigil.yaml",
        "containers:\n  engines: [podman]\n",
    );

    let config = loaded(&directory);

    assert_eq!(
        config.of_the_module("containers-engines"),
        json!({"engines": ["podman"]})
    );
    assert_eq!(config.of_the_module("containers"), serde_json::Value::Null);
}

#[test]
fn the_engines_written_under_their_former_key_and_their_own_are_refused() {
    let directory = bench("former-engines-twice");
    written(
        &directory,
        "vigil.yaml",
        "containers:\n  engines: [podman]\ncontainers-engines:\n  engines: [docker]\n",
    );

    let error = refused(&directory);

    assert!(error.cause.contains("containers-engines"), "{error}");
}

#[test]
fn the_former_name_is_not_read_in_the_collectors_directory_where_nothing_was_ever_called_that() {
    let directory = apart("no-alias", &[("ports.yaml", "ports:\n")]);

    let error = refused(&directory);

    assert!(error.cause.contains("ports"), "{error}");
    assert!(error.cause.contains("network"), "{error}");
}
