use std::collections::BTreeSet;

use vigil_model::Snapshot;

use super::verdict::judged;
use crate::fixture;

fn every_kind_raised_when_the_whole_sample_arrives_at_once() -> BTreeSet<String> {
    let after = fixture::engines();
    let mut before = Snapshot::new(after.source.clone(), after.taken_at.clone());
    for (key, item) in &after.items {
        if key.contains("|engine|") {
            before.items.insert(key.clone(), item.clone());
        }
    }

    judged(&before, &after)
        .into_iter()
        .map(|finding| finding.kind.to_string())
        .collect()
}

#[test]
fn every_dangerous_setting_in_the_sample_is_reported_when_it_arrives() {
    let raised = every_kind_raised_when_the_whole_sample_arrives_at_once();

    for kind in [
        "container.network.host_mode",
        "container.volume.host_mount",
        "container.image.untagged_in_use",
        "container.registry.insecure",
        "container.image.new",
        "container.volume.new",
        "container.network.new",
        "container.project.new",
        "container.pod.new",
        "container.secret.new",
    ] {
        assert!(raised.contains(kind), "{kind} is not raised: {raised:?}");
    }
}

#[test]
fn a_privileged_service_is_named_by_the_vocabulary_and_raised_by_no_rule_because_no_list_says_it() {
    let raised = every_kind_raised_when_the_whole_sample_arrives_at_once();

    assert!(
        !raised.contains("container.project.privileged_service"),
        "`docker ps` and `podman ps` do not print whether a container runs with --privileged; \
         only `inspect` does, and `inspect` also prints the environment, which this agent \
         never asks for. A rule claiming to know is a rule guessing"
    );
    assert!(
        !raised.contains("container.image_unknown"),
        "every image a container of the sample runs is an image its engine lists, and an image \
         the engine does not know is only the race between two of the dump's commands"
    );
}
