use vigil_module::{Module, Settings};

use super::Processes;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_a_program_walks_to_the_row_of_the_program_it_is_about() {
    assert_eq!(
        Processes.row_of("process|exec|/bin/sh|www-data"),
        Some("exec|/bin/sh|www-data".to_string())
    );
    assert!(!Processes.raised("run|alice|/usr/bin/nc"));
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Processes.name(), "processes");
    assert_eq!(Processes.every_seconds(), 30);
    assert!(!Processes.rules(&Settings::plain(at_noon)).is_empty());
    assert!(Processes.section().is_some());
}

#[test]
fn this_module_and_the_launches_share_one_section_of_the_console() {
    let section = Processes.section().expect("a section");

    assert_eq!(
        section.name(),
        "programs",
        "what runs now and what was run are one screen with two lists, and the two modules \
         that read them both name that screen"
    );
}
