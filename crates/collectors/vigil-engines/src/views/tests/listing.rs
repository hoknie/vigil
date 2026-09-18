use vigil_view::{Facet, Pane, Room, Section, Showing};

use super::super::WhatTheEnginesHold;
use crate::fixture::{self, CONTAINER_MOUNTING_ETC, HOST_NETWORK_CONTAINER, UNTAGGED_IMAGE};

fn pane(group: &str, name: &str) -> Box<dyn Pane> {
    WhatTheEnginesHold
        .panes()
        .into_iter()
        .find(|pane| pane.belongs_to() == Some(group) && pane.name() == name)
        .unwrap_or_else(|| panic!("no list {name} under {group}"))
}

fn drawn(pane: &dyn Pane, key: &str, room: u16) -> String {
    let reading = fixture::engines();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key == key)
        .unwrap_or_else(|| panic!("{key} is not listed"));
    pane.cells(&reading, &row, Room::of(room))
        .into_iter()
        .map(|cell| cell.text)
        .collect::<Vec<String>>()
        .join(" | ")
}

#[test]
fn a_list_holds_the_rows_of_its_own_engine_and_subject_and_no_other() {
    let reading = fixture::engines();

    for pane in WhatTheEnginesHold.panes() {
        let prefix = format!(
            "{}|",
            pane.belongs_to().expect("every list belongs to an engine")
        );
        for row in pane.rows(&reading, &Showing::default()) {
            assert!(
                row.key.starts_with(&prefix),
                "{} lists {}",
                pane.name(),
                row.key
            );
        }
    }
    assert_eq!(
        pane("docker", "images")
            .rows(&reading, &Showing::default())
            .len(),
        3
    );
    assert_eq!(
        pane("podman", "secrets")
            .rows(&reading, &Showing::default())
            .len(),
        1
    );
}

#[test]
fn a_container_is_drawn_with_its_image_its_networks_and_the_paths_of_this_host_it_mounts() {
    let web = drawn(
        pane("docker", "containers").as_ref(),
        &format!("docker|container|{CONTAINER_MOUNTING_ETC}"),
        80,
    );
    let agent = drawn(
        pane("docker", "containers").as_ref(),
        &format!("docker|container|{HOST_NETWORK_CONTAINER}"),
        160,
    );

    assert_eq!(web, "shop-web-1 | nginx:1.27-alpine | shop_default | 1");
    assert_eq!(
        agent,
        "metrics-agent-1 | metrics | 5f0c1ad8b29e | host | \u{2014} | 2"
    );
}

#[test]
fn an_image_is_drawn_with_its_tags_and_the_number_of_containers_that_run_it() {
    let images = pane("docker", "images");

    assert_eq!(
        drawn(
            images.as_ref(),
            &format!("docker|image|{UNTAGGED_IMAGE}"),
            80
        ),
        "<none> | sha256:5f0c1ad8b29e | 1.02GB | 1",
        "the untagged image in the sample is the one the metrics agent runs, by its id"
    );
    assert_eq!(
        drawn(images.as_ref(), "docker|image|sha256:18ad9bdc4c87", 160),
        "nginx:1.27-alpine | sha256:18ad9bdc4c87 | sha256:9a7f6e5d4c3b | 78.5MB | 1"
    );
}

#[test]
fn a_row_with_something_to_look_at_is_listed_before_the_rows_without() {
    let reading = fixture::engines();
    let rows = pane("docker", "containers").rows(&reading, &Showing::default());

    assert_eq!(
        rows.iter().map(|row| row.key.as_str()).collect::<Vec<_>>(),
        vec![
            "docker|container|metrics-agent-1",
            "docker|container|shop-web-1",
            "docker|container|shop-api-1"
        ],
        "as the agent sends it, the containers on the host network or holding a path of this \
         host come first: they are why a reader opens this list"
    );
}

#[test]
fn a_list_narrowed_to_a_compose_project_holds_that_project_and_says_so_in_its_footer() {
    let reading = fixture::engines();
    let containers = pane("docker", "containers");
    let first = containers
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.ends_with("shop-web-1"))
        .expect("a container of shop");
    let offered = containers.facets(&reading, &first);
    assert_eq!(offered, vec![Facet::new("project", "shop")]);

    let showing = Showing::default().narrowing(&offered);
    let rows = containers.rows(&reading, &showing);
    let footer = containers.tally(&reading, &showing, rows.len());

    assert_eq!(rows.len(), 2);
    assert!(
        footer.starts_with("2 of 3 container(s) of docker"),
        "{footer}"
    );
    assert!(footer.contains("only project shop"), "{footer}");
    assert!(
        pane("docker", "images").facets(&reading, &first).is_empty(),
        "an image belongs to no project, and a list offering to narrow by one would narrow to \
         nothing"
    );
}

#[test]
fn the_footer_names_the_engine_its_version_and_what_in_the_list_is_worth_a_look() {
    let reading = fixture::engines();
    let containers = pane("docker", "containers");
    let footer = containers.tally(&reading, &Showing::default(), 3);

    assert_eq!(
        footer,
        "3 container(s) of docker, read at 09:00:01 \u{b7} docker 27.1.1, overlay2, rootful \u{b7} \
         1 on the host network \u{b7} 2 mounting paths of this host \u{b7} 1 running an untagged image"
    );
    let podman = pane("podman", "networks").tally(&reading, &Showing::default(), 2);
    assert!(
        podman.contains("podman 5.1.1, overlay, rootless"),
        "{podman}"
    );
    assert!(podman.contains("1 internal"), "{podman}");
}

#[test]
fn the_detail_of_a_row_says_every_field_what_it_means_and_the_key_its_findings_are_raised_under() {
    let reading = fixture::engines();
    let volumes = pane("podman", "volumes");
    let row = vigil_view::RowKey::of("podman|volume|etc_backup");

    let said = format!("{:?}", volumes.detail(&reading, &row, 80));

    for wanted in [
        "binds",
        "/etc",
        "WHAT TO LOOK AT",
        "It binds /etc of this host.",
        "WHAT EACH FIELD MEANS",
        "the path of this host a bind volume points at",
        "Key(\"engine|podman|volume|etc_backup\")",
    ] {
        assert!(said.contains(wanted), "{wanted} is not in {said}");
    }
}
