use vigil_module::{Module, Settings};

use super::Containers;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_a_container_walks_to_the_row_of_the_container_it_is_about() {
    assert_eq!(
        Containers.row_of("container|privileged|3ab1c0f2d4e5"),
        Some("3ab1c0f2d4e5".to_string()),
        "the finding names what happened and then the container; the reading is keyed by \
         the container alone"
    );
    assert!(!Containers.raised("port.listen|tcp|0.0.0.0:443"));
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Containers.name(), "containers");
    assert_eq!(Containers.every_seconds(), 60);
    assert_eq!(Containers.unit(), None);
    assert!(!Containers.rules(&Settings::plain(at_noon)).is_empty());
    assert!(Containers.section().is_some());
}
