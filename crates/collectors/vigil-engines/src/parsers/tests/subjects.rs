use serde_json::json;

use crate::fixture::{self, row};
use crate::types::{Engine, Subject};

#[test]
fn an_image_with_no_tag_on_it_is_a_row_that_says_so_rather_than_a_row_with_an_empty_name() {
    let reading = fixture::engines();

    let untagged = row(
        &reading,
        Engine::Docker,
        Subject::Image,
        fixture::UNTAGGED_IMAGE,
    );
    let tagged = row(
        &reading,
        Engine::Docker,
        Subject::Image,
        fixture::TAGGED_IMAGE,
    );

    assert_eq!(untagged["untagged"], json!(true));
    assert_eq!(untagged["tags"], json!([]));
    assert_eq!(untagged["digest"], json!(null));
    assert_eq!(tagged["untagged"], json!(false));
    assert_eq!(tagged["tags"], json!(["nginx:1.27-alpine"]));
    assert!(
        tagged["digest"]
            .as_str()
            .is_some_and(|said| said.starts_with("sha256:"))
    );
}

#[test]
fn one_image_under_two_tags_is_one_row_holding_both_and_not_two_rows_of_one_image() {
    let mut twice = fixture::docker::dump();
    let answer = twice.asked.get_mut("image").expect("asked for");
    answer.printed.push_str(
        r#"{"Digest":"<none>","ID":"18ad9bdc4c87","Repository":"nginx","Size":"78.5MB","Tag":"latest"}"#,
    );
    answer.printed.push('\n');

    let reading = fixture::read(&twice, &fixture::podman::dump());
    let image = row(
        &reading,
        Engine::Docker,
        Subject::Image,
        fixture::TAGGED_IMAGE,
    );

    assert_eq!(
        image["tags"],
        json!(["nginx:1.27-alpine", "nginx:latest"]),
        "docker prints one line per tag, so a row per line would report the same image twice \
         and a retag would read as one image gone and another arrived"
    );
}

#[test]
fn a_volume_bound_to_a_directory_of_this_host_names_that_directory_in_the_row() {
    let volume = row(
        &fixture::engines(),
        Engine::Podman,
        Subject::Volume,
        fixture::VOLUME_FROM_ETC,
    );

    assert_eq!(volume["device"], json!("/etc"));
    assert_eq!(volume["driver"], json!("local"));
}

#[test]
fn a_network_with_a_subnet_carries_it_and_a_network_the_engine_prints_none_for_carries_an_empty_list()
 {
    let reading = fixture::engines();

    let bridge = row(
        &reading,
        Engine::Podman,
        Subject::Network,
        fixture::BRIDGE_WITH_A_SUBNET,
    );
    let docker_bridge = row(&reading, Engine::Docker, Subject::Network, "bridge");

    assert_eq!(bridge["subnets"], json!(["10.89.0.0/24"]));
    assert_eq!(bridge["internal"], json!(true));
    assert_eq!(
        docker_bridge["subnets"],
        json!([]),
        "`docker network ls` prints no subnet and this dump asks it nothing else, so the row \
         says the list is empty here rather than inventing one"
    );
}

#[test]
fn a_container_on_the_network_of_this_host_says_so_in_a_field_of_its_own() {
    let reading = fixture::engines();

    let agent = row(
        &reading,
        Engine::Docker,
        Subject::Container,
        fixture::HOST_NETWORK_CONTAINER,
    );
    let web = row(
        &reading,
        Engine::Docker,
        Subject::Container,
        fixture::CONTAINER_MOUNTING_ETC,
    );

    assert_eq!(agent["host_network"], json!(true));
    assert_eq!(agent["image"], json!("5f0c1ad8b29e"));
    assert_eq!(web["host_network"], json!(false));
    assert_eq!(web["mounts"], json!(["/etc", "shop_database"]));
    assert_eq!(web["ports"], json!(["0.0.0.0:443->443/tcp"]));
}

#[test]
fn a_published_port_reads_the_same_whichever_engine_printed_it() {
    let shell = row(
        &fixture::engines(),
        Engine::Podman,
        Subject::Container,
        "tools-shell",
    );

    assert_eq!(shell["ports"], json!(["127.0.0.1:2222->2222/tcp"]));
}

#[test]
fn every_compose_project_on_this_host_is_a_row_naming_its_services_and_its_file() {
    let reading = fixture::engines();

    for name in fixture::PROJECTS {
        let project = row(&reading, Engine::Docker, Subject::Project, name);
        assert_eq!(project["name"], json!(name));
        assert!(
            project["configuration_files"]
                .as_array()
                .is_some_and(|files| !files.is_empty()),
            "{name} names no compose file, and a project nobody can find the file of is a \
             row a reader can do nothing with"
        );
    }

    let shop = row(&reading, Engine::Docker, Subject::Project, "shop");
    assert_eq!(shop["services"], json!(["api", "web"]));
    assert_eq!(shop["containers"], json!(["shop-api-1", "shop-web-1"]));
    assert_eq!(shop["working_directory"], json!("/srv/shop"));
}

#[test]
fn a_pod_names_the_containers_in_it_and_never_what_any_of_them_is_doing_now() {
    let pod = row(
        &fixture::engines(),
        Engine::Podman,
        Subject::Pod,
        fixture::POD,
    );

    assert_eq!(pod["containers"], json!(["tools-infra", "tools-shell"]));
    assert_eq!(pod["networks"], json!(["podman1"]));
    let written = pod.to_string();
    assert!(
        !written.contains("Running") && !written.contains("Up "),
        "a pod's status moves with the containers in it: {written}"
    );
}

#[test]
fn a_registry_the_engine_may_reach_without_tls_is_a_row_that_says_insecure() {
    let reading = fixture::engines();

    let docker = row(
        &reading,
        Engine::Docker,
        Subject::Registry,
        fixture::INSECURE_REGISTRY,
    );
    let podman = row(
        &reading,
        Engine::Podman,
        Subject::Registry,
        fixture::INSECURE_REGISTRY,
    );

    assert_eq!(docker["insecure"], json!(true));
    assert_eq!(docker["from"], json!("/etc/docker/daemon.json"));
    assert_eq!(podman["insecure"], json!(true));
    assert_eq!(podman["from"], json!("/etc/containers/registries.conf"));
}

#[test]
fn each_engine_has_one_row_about_itself_whether_or_not_it_is_installed_here() {
    let both = fixture::engines();
    let one = fixture::only_docker();

    assert_eq!(
        row(&both, Engine::Podman, Subject::Engine, "podman")["present"],
        json!(true)
    );
    assert_eq!(
        row(&one, Engine::Podman, Subject::Engine, "podman")["present"],
        json!(false),
        "a host where podman was uninstalled must show that as a change, and a row that \
         disappears from the reading says nothing about why"
    );
    assert_eq!(
        row(&both, Engine::Podman, Subject::Engine, "podman")["rootless"],
        json!(true)
    );
    assert_eq!(
        row(&both, Engine::Docker, Subject::Engine, "docker")["version"],
        json!("27.1.1")
    );
}
