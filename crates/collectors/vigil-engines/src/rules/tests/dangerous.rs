use serde_json::json;
use vigil_model::Severity;

use super::verdict::{altered, judged, judged_under, said, silenced_everything, with, without};
use crate::fixture::{
    self, CONTAINER_MOUNTING_ETC, HOST_NETWORK_CONTAINER, INSECURE_REGISTRY, VOLUME_FROM_ETC,
};
use crate::types::{Engine, Subject};

fn arrived(engine: Engine, subject: Subject, id: &str) -> Vec<(String, String)> {
    let after = fixture::engines();
    said(&judged(&without(&after, engine, subject, id), &after))
}

#[test]
fn a_container_that_arrives_on_the_host_network_running_an_untagged_image_is_two_findings() {
    let findings = arrived(Engine::Docker, Subject::Container, HOST_NETWORK_CONTAINER);
    let key = format!("engine|docker|container|{HOST_NETWORK_CONTAINER}");

    assert!(
        findings.contains(&("container.network.host_mode".to_string(), key.clone())),
        "{findings:?}"
    );
    assert!(
        findings.contains(&("container.image.untagged_in_use".to_string(), key.clone())),
        "{findings:?}"
    );
    assert!(
        findings.contains(&("container.volume.host_mount".to_string(), key)),
        "the agent in the sample mounts /proc and /sys of this host: {findings:?}"
    );
}

#[test]
fn a_container_already_on_the_host_network_is_not_reported_again_when_something_else_moves() {
    let before = fixture::engines();
    let after = altered(
        &before,
        Engine::Docker,
        Subject::Container,
        HOST_NETWORK_CONTAINER,
        "ports",
        json!(["9100/tcp"]),
    );

    assert!(
        judged(&before, &after).is_empty(),
        "{:?}",
        said(&judged(&before, &after))
    );
}

#[test]
fn a_volume_bound_to_the_configuration_of_this_host_is_high_and_one_bound_to_its_root_critical() {
    let before = without(
        &fixture::engines(),
        Engine::Podman,
        Subject::Volume,
        VOLUME_FROM_ETC,
    );
    let etc = fixture::engines();
    let root = altered(
        &etc,
        Engine::Podman,
        Subject::Volume,
        VOLUME_FROM_ETC,
        "device",
        json!("/"),
    );

    let at_etc = judged(&before, &etc);
    let host_mount = at_etc
        .iter()
        .find(|finding| finding.kind.as_str() == "container.volume.host_mount")
        .expect("a volume holding /etc");
    assert_eq!(host_mount.finding_key, "engine|podman|volume|etc_backup");
    assert_eq!(host_mount.severity, Severity::High);
    assert_eq!(host_mount.subject.object, "volume");

    let at_root = judged(&before, &root);
    assert!(
        at_root.iter().any(
            |finding| finding.kind.as_str() == "container.volume.host_mount"
                && finding.severity == Severity::Critical
        ),
        "{:?}",
        said(&at_root)
    );
}

#[test]
fn a_docker_container_mounting_etc_is_held_of_this_host_and_a_podman_one_is_not_judged() {
    let docker = arrived(Engine::Docker, Subject::Container, CONTAINER_MOUNTING_ETC);
    let podman = arrived(Engine::Podman, Subject::Container, "tools-shell");

    assert!(
        docker.contains(&(
            "container.volume.host_mount".to_string(),
            format!("engine|docker|container|{CONTAINER_MOUNTING_ETC}")
        )),
        "{docker:?}"
    );
    assert!(
        podman.is_empty(),
        "podman's `ps` prints where a mount lands inside the container and not where it comes \
         from, so /etc there is the container's own; the volume rule judges what podman binds: \
         {podman:?}"
    );
}

#[test]
fn a_mount_path_the_engine_cut_short_is_judged_only_where_every_ending_is_of_this_host() {
    let before = fixture::engines();
    let judged_with = |mounts: serde_json::Value| {
        let after = altered(
            &before,
            Engine::Docker,
            Subject::Container,
            "shop-api-1",
            "mounts",
            mounts,
        );
        said(&judged(&before, &after))
    };

    assert_eq!(judged_with(json!(["/var/lib/docke\u{2026}"])), Vec::new());
    assert_eq!(judged_with(json!(["/etc/letsencr\u{2026}"])).len(), 1);
}

#[test]
fn a_registry_this_host_may_reach_without_tls_is_high_once_and_not_again() {
    let before = without(
        &fixture::engines(),
        Engine::Docker,
        Subject::Registry,
        INSECURE_REGISTRY,
    );
    let after = fixture::engines();

    let findings = judged(&before, &after);
    assert_eq!(
        said(&findings),
        vec![(
            "container.registry.insecure".to_string(),
            format!("engine|docker|registry|{INSECURE_REGISTRY}")
        )]
    );
    assert_eq!(findings[0].severity, Severity::High);

    let turned = altered(
        &after,
        Engine::Docker,
        Subject::Registry,
        "mirror.example.com",
        "insecure",
        json!(true),
    );
    assert_eq!(said(&judged(&after, &turned)).len(), 1);
    let still = altered(
        &after,
        Engine::Docker,
        Subject::Registry,
        INSECURE_REGISTRY,
        "role",
        json!("mirror"),
    );
    assert!(judged(&after, &still).is_empty());
}

#[test]
fn a_dangerous_setting_is_reported_whatever_the_switches_in_report_say() {
    let before = without(
        &fixture::engines(),
        Engine::Docker,
        Subject::Registry,
        INSECURE_REGISTRY,
    );

    assert_eq!(
        said(&judged_under(
            &silenced_everything(),
            &before,
            &fixture::engines()
        ))
        .len(),
        1,
        "the switches are for hosts whose images change on every deploy; a registry without \
         TLS is not a deploy"
    );
}

#[test]
fn a_label_hidden_on_the_agent_is_listed_by_the_finding_and_its_value_is_nowhere_in_it() {
    let findings = judged(
        &without(
            &fixture::engines(),
            Engine::Docker,
            Subject::Container,
            CONTAINER_MOUNTING_ETC,
        ),
        &fixture::engines(),
    );
    let finding = findings.first().expect("a container mounting /etc");

    assert_eq!(finding.redacted, vec!["/after/labels/com.shop.api-token"]);
    let written = serde_json::to_string(&finding).expect("plain data");
    assert!(!written.contains("s.9a7f6e5d4c3b"), "{written}");
}

#[test]
fn a_container_whose_image_lost_its_tag_between_two_readings_is_reported_then() {
    let before = fixture::engines();
    let after = with(&before, Engine::Docker, Subject::Container, "shop-api-1", {
        let mut item = fixture::row(&before, Engine::Docker, Subject::Container, "shop-api-1");
        item["image"] = json!("c4b2e1f09a33");
        item
    });

    assert_eq!(
        said(&judged(&before, &after)),
        vec![(
            "container.image.untagged_in_use".to_string(),
            "engine|docker|container|shop-api-1".to_string()
        )]
    );
}
