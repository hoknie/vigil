use vigil_model::Change;

use super::verdict::containers;
use crate::fixture;

const ORDINARY: &str = "00000000a80425fb";

const EVERYTHING: &str = "000001ffffffffff";

#[test]
fn exactly_one_container_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "a container was started with --privileged",
            Change::Added {
                key: "container|3ab1c0f2d4e5".into(),
                after: fixture::container("/usr/local/bin/agent", EVERYTHING, &[]),
            },
            "container_privileged",
            "container.privileged",
        ),
        (
            "a container was started holding the root of this host",
            Change::Added {
                key: "container|3ab1c0f2d4e5".into(),
                after: fixture::container("/usr/sbin/nginx", ORDINARY, &["/"]),
            },
            "container_host_mount",
            "container.host_mount",
        ),
        (
            "a container was started holding the socket of its own runtime",
            Change::Added {
                key: "container|3ab1c0f2d4e5".into(),
                after: fixture::container("/usr/local/bin/agent", ORDINARY, &["/run/docker.sock"]),
            },
            "container_docker_socket_exposed",
            "container.docker_socket_exposed",
        ),
        (
            "somebody chmodded the socket of the runtime",
            Change::Changed {
                key: "container-socket|/run/docker.sock".into(),
                before: fixture::runtime_socket("/run/docker.sock", "0660"),
                after: fixture::runtime_socket("/run/docker.sock", "0666"),
            },
            "container_docker_socket_exposed",
            "container.docker_socket_exposed",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = containers(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn a_container_that_is_privileged_and_holds_the_host_is_two_findings_and_not_one() {
    let change = Change::Added {
        key: "container|3ab1c0f2d4e5".into(),
        after: fixture::container("/usr/local/bin/agent", EVERYTHING, &["/"]),
    };

    let both = containers(&change);

    assert_eq!(both.len(), 2, "{both:?}");
    assert!(both.contains(&(
        "container_privileged".to_string(),
        "container.privileged".to_string()
    )));
    assert!(both.contains(&(
        "container_host_mount".to_string(),
        "container.host_mount".to_string()
    )));
}

#[test]
fn a_container_a_runtime_started_the_way_it_starts_every_container_says_nothing() {
    let ordinary = Change::Added {
        key: "container|3ab1c0f2d4e5".into(),
        after: fixture::container(
            "/usr/sbin/nginx",
            ORDINARY,
            &[
                "/var/lib/docker/containers/3ab1/resolv.conf",
                "/var/lib/docker/volumes/site/_data",
            ],
        ),
    };

    assert!(
        containers(&ordinary).is_empty(),
        "every container on every host looks like this one, and a product that says something \
         about each of them is turned off in a week"
    );
}

#[test]
fn a_container_that_went_away_is_not_a_finding_about_anything() {
    let change = Change::Removed {
        key: "container|3ab1c0f2d4e5".into(),
        before: fixture::container("/usr/local/bin/agent", EVERYTHING, &["/"]),
    };

    assert!(
        containers(&change).is_empty(),
        "this vocabulary has no kind for a container that stopped, and inventing one in a rule \
         would be a change to a published contract"
    );
}

#[test]
fn the_image_a_container_was_started_from_is_named_by_no_rule_here() {
    let change = Change::Added {
        key: "container|3ab1c0f2d4e5".into(),
        after: fixture::container("/usr/sbin/nginx", ORDINARY, &[]),
    };

    assert!(
        containers(&change).is_empty(),
        "container.image_unknown is in the vocabulary and nothing produces it: /proc does not \
         name the image a process was started from, and the engines reading, which does, lists \
         every image its containers run from, so an image it does not know is only the race \
         between two of its commands. A rule guessing at it would be worse than the silence"
    );
}
