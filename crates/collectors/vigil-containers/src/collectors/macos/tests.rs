use vigil_collect::{CollectError, Collector, Health};
use vigil_module::{Module, Settings};

use super::ContainersCollector;
use super::collector::IN_A_VIRTUAL_MACHINE;
use crate::Containers;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-19T12:00:00.000Z".to_string()
}

#[test]
fn containers_on_a_mac_are_unavailable_and_the_reason_names_the_collector_that_does_read_them() {
    let collector = ContainersCollector::new(at_noon);

    assert_eq!(collector.name(), "containers");
    assert_eq!(
        collector.available(),
        Health::Unavailable(IN_A_VIRTUAL_MACHINE.to_string())
    );
    assert!(IN_A_VIRTUAL_MACHINE.contains("containers-engines"));
}

#[test]
fn a_mac_is_never_read_as_a_host_running_no_containers() {
    match ContainersCollector::new(at_noon).collect() {
        Err(CollectError::Absent(why)) => assert_eq!(why, IN_A_VIRTUAL_MACHINE),
        other => {
            panic!("an empty reading would be stored as a host where no container runs: {other:?}")
        }
    }
}

#[test]
fn the_module_hands_the_daemon_a_collector_that_says_why_rather_than_refusing_to_build_one() {
    let collector = Containers
        .collector(&Settings::plain(at_noon))
        .expect("a collector that answers unavailable");

    assert!(matches!(collector.available(), Health::Unavailable(_)));
}
