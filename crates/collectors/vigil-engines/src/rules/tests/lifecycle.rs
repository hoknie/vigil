use serde_json::json;
use vigil_model::{KnownKind, Severity};

use super::verdict::{altered, judged, judged_under, said, silenced_everything, with, without};
use crate::fixture::{self, POD, SECRET, TAGGED_IMAGE, VOLUME_FROM_ETC};
use crate::types::{Engine, Report, Subject};

fn one(kind: &str, key: &str) -> Vec<(String, String)> {
    vec![(kind.to_string(), key.to_string())]
}

#[test]
fn a_host_whose_engines_hold_what_they_held_says_nothing() {
    let reading = fixture::engines();

    assert!(judged(&reading, &reading).is_empty());
}

#[test]
fn an_image_pulled_between_two_readings_is_a_new_image_keyed_by_its_engine_and_its_id() {
    let before = fixture::engines();
    let after = with(
        &before,
        Engine::Docker,
        Subject::Image,
        "sha256:77aa00bb11cc",
        json!({"subject": "image", "id": "sha256:77aa00bb11cc", "tags": ["redis:7"], "untagged": false, "digest": null, "size": "40MB"}),
    );

    let findings = judged(&before, &after);

    assert_eq!(
        said(&findings),
        one(
            "container.image.new",
            "engine|docker|image|sha256:77aa00bb11cc"
        )
    );
    assert_eq!(findings[0].severity, Severity::Low);
    assert!(
        findings[0].title.contains("redis:7"),
        "{}",
        findings[0].title
    );
}

#[test]
fn a_volume_that_went_is_keyed_as_the_one_that_came_so_that_the_second_closes_the_first() {
    let before = fixture::engines();
    let after = without(&before, Engine::Podman, Subject::Volume, VOLUME_FROM_ETC);

    let gone = judged(&before, &after);
    let back = judged(&after, &before);

    assert_eq!(gone.len(), 1, "{:?}", said(&gone));
    assert_eq!(gone[0].kind.as_str(), "container.volume.removed");
    assert_eq!(
        gone[0].finding_key,
        back.iter()
            .find(|finding| finding.kind.as_str() == "container.volume.new")
            .expect("the volume coming back is a new volume")
            .finding_key,
        "a removal closes the finding about the appearance only under the same key"
    );
    assert_eq!(
        KnownKind::ContainerVolumeRemoved.resolves(),
        Some(KnownKind::ContainerVolumeNew)
    );
}

#[test]
fn a_network_is_changed_by_its_subnets_its_driver_and_whether_it_is_internal_and_by_nothing_else() {
    let before = fixture::engines();
    let moved = altered(
        &before,
        Engine::Podman,
        Subject::Network,
        "podman1",
        "subnets",
        json!(["10.90.0.0/24"]),
    );
    let opened = altered(
        &before,
        Engine::Podman,
        Subject::Network,
        "podman1",
        "internal",
        json!(false),
    );
    let renumbered = altered(
        &before,
        Engine::Podman,
        Subject::Network,
        "podman1",
        "id",
        json!("ffff"),
    );

    let findings = judged(&before, &moved);
    assert_eq!(
        said(&findings),
        one("container.network.changed", "engine|podman|network|podman1")
    );
    assert_eq!(findings[0].severity, Severity::Medium);
    assert!(
        findings[0]
            .evidence
            .iter()
            .any(|one| one.value.contains("10.89.0.0/24") && one.value.contains("10.90.0.0/24")),
        "the subnet it had is the half a reader compares against: {:?}",
        findings[0].evidence
    );
    assert_eq!(said(&judged(&before, &opened)).len(), 1);
    assert!(
        judged(&before, &renumbered).is_empty(),
        "an id an engine gives a network it recreated with the same settings is not a \
         setting of the network"
    );
}

#[test]
fn a_volume_is_changed_by_its_driver_its_point_and_the_path_it_binds() {
    let before = fixture::engines();

    for (field, value) in [
        ("driver", json!("nfs")),
        ("mountpoint", json!("/srv/elsewhere")),
        ("device", json!("/opt/backup")),
    ] {
        let after = altered(
            &before,
            Engine::Docker,
            Subject::Volume,
            "shop_database",
            field,
            value,
        );
        assert_eq!(
            said(&judged(&before, &after)),
            one(
                "container.volume.changed",
                "engine|docker|volume|shop_database"
            ),
            "{field}"
        );
    }
    let relabelled = altered(
        &before,
        Engine::Docker,
        Subject::Volume,
        "shop_database",
        "scope",
        json!("global"),
    );
    assert!(judged(&before, &relabelled).is_empty());
}

#[test]
fn a_compose_project_changes_when_it_gains_or_loses_a_service_and_not_when_it_restarts() {
    let before = fixture::engines();
    let grown = altered(
        &before,
        Engine::Docker,
        Subject::Project,
        "shop",
        "services",
        json!(["api", "web", "worker"]),
    );
    let recreated = altered(
        &before,
        Engine::Docker,
        Subject::Project,
        "shop",
        "containers",
        json!(["shop-api-2", "shop-web-2"]),
    );

    let findings = judged(&before, &grown);
    assert_eq!(
        said(&findings),
        one("container.project.changed", "engine|docker|project|shop")
    );
    assert!(
        findings[0].title.contains("services"),
        "{}",
        findings[0].title
    );
    assert!(
        judged(&before, &recreated).is_empty(),
        "compose names a recreated container anew on every `up`, and the project running the \
         same services is the same project"
    );
}

#[test]
fn a_pod_changes_with_the_containers_it_holds_and_a_secret_when_it_was_replaced() {
    let before = fixture::engines();
    let pod = altered(
        &before,
        Engine::Podman,
        Subject::Pod,
        POD,
        "containers",
        json!(["tools-infra"]),
    );
    let secret = altered(
        &before,
        Engine::Podman,
        Subject::Secret,
        SECRET,
        "updated_at",
        json!("2026-09-18T06:00:00Z"),
    );

    assert_eq!(
        said(&judged(&before, &pod)),
        one("container.pod.changed", "engine|podman|pod|tools")
    );
    let replaced = judged(&before, &secret);
    assert_eq!(
        said(&replaced),
        one(
            "container.secret.changed",
            "engine|podman|secret|shop-database-password"
        )
    );
    assert_eq!(replaced[0].severity, Severity::Medium);
    assert_eq!(
        replaced[0].redacted,
        vec!["/before/value", "/after/value"],
        "a secret's value is never read, and the finding says it is hidden rather than empty"
    );
}

#[test]
fn a_switch_turned_off_in_report_silences_its_subject_and_leaves_the_others_speaking() {
    let before = fixture::engines();
    let after = without(
        &without(&before, Engine::Docker, Subject::Image, TAGGED_IMAGE),
        Engine::Podman,
        Subject::Secret,
        SECRET,
    );
    let without_images = Report {
        images: false,
        ..Report::default()
    };

    let kinds: Vec<String> = said(&judged_under(&without_images, &before, &after))
        .into_iter()
        .map(|(kind, _)| kind)
        .collect();

    assert_eq!(kinds, vec!["container.secret.removed"]);
    assert!(judged_under(&silenced_everything(), &before, &after).is_empty());
}
