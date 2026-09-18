use vigil_module::{Module, Settings};

use super::Engines;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-17T12:00:00.000Z".to_string()
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Engines.name(), "containers-engines");
    assert_eq!(Engines.every_seconds(), 120);
    assert_eq!(Engines.unit(), Some("vigil-containers.timer"));
    assert_eq!(Engines.settings_key(), Some("containers-engines"));
}

#[test]
fn the_engines_are_drawn_as_groups_of_the_containers_screen_and_not_as_a_screen_of_their_own() {
    let section = Engines
        .section()
        .expect("the engines declare what they draw");

    assert_eq!(
        section.name(),
        "containers",
        "a reader looking for a container should not have to know whether /proc or an engine \
         saw it, so the engines are the docker and podman groups of the one containers screen"
    );
    assert_eq!(section.groups(), vec!["docker", "podman"]);
}

#[test]
fn a_finding_of_the_engines_walks_to_the_row_of_the_reading_it_is_about() {
    assert_eq!(Engines.families(), &["engine"]);
    assert!(Engines.raised("engine|docker|image|sha256:18ad9bdc4c87"));
    assert!(
        !Engines.raised("container|privileged|3ab1c0f2d4e5"),
        "the /proc reading of the containers raises its own family, and a finding about a \
         running process is walked to that screen's row, not to an engine's"
    );
    assert_eq!(
        Engines.row_of("engine|docker|image|sha256:18ad9bdc4c87"),
        Some("docker|image|sha256:18ad9bdc4c87".to_string()),
        "the key after the family is the reading's own, so the console finds the row by it"
    );
}

#[test]
fn the_period_this_module_reads_at_is_the_period_the_dump_is_written_at() {
    assert_eq!(
        u64::from(Engines.every_seconds()) * 2,
        crate::types::Watching::default().stale_after_seconds(),
        "the timer and the collector are two halves of one reading: a collector reading \
         faster than the dump is written finds the same document over and over, and one \
         reading slower leaves the console showing a host as it was two periods ago"
    );
}

#[test]
fn a_block_naming_an_engine_this_build_does_not_read_stops_the_daemon_at_the_door() {
    let refusal = Engines
        .check(&Settings::of(
            at_noon,
            "containers-engines",
            serde_json::json!({"engines": ["docker", "containerd"]}),
        ))
        .expect_err("must not be accepted");

    assert!(refusal.contains("containerd"), "{refusal}");
}

#[test]
fn a_configuration_that_names_nothing_at_all_is_a_module_on_its_own_values() {
    assert!(Engines.check(&Settings::plain(at_noon)).is_ok());
    assert!(
        Engines
            .check(&Settings::of(
                at_noon,
                "containers-engines",
                serde_json::json!({})
            ))
            .is_ok()
    );
}

#[test]
fn every_switch_in_report_takes_one_subject_out_and_the_dangerous_settings_never() {
    let everything = Engines.rules(&Settings::plain(at_noon)).len();
    let quiet = Engines
        .rules(&Settings::of(
            at_noon,
            "containers-engines",
            serde_json::json!({"report": {
                "images": false, "volumes": false, "networks": false,
                "projects": false, "pods": false, "secrets": false,
            }}),
        ))
        .len();

    assert_eq!(
        everything - quiet,
        7,
        "six subjects that come and go, and the rule about a tag that moved, which is an \
         image changing"
    );
    assert_eq!(
        quiet, 5,
        "what did not answer is claimed before any rule reads it, and the four dangerous \
         settings are reported whatever the switches say"
    );
}
