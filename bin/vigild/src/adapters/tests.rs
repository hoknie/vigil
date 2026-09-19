use vigil_collect::{Collector, Health};

use super::Unported;

#[test]
fn a_subject_this_build_cannot_read_here_is_unavailable_and_says_why_in_the_modules_words() {
    let unported = Unported::new("launches", "what people run is read from auditd");

    assert_eq!(unported.name(), "launches");
    match unported.available() {
        Health::Unavailable(why) => {
            assert!(why.contains("what people run is read from auditd"), "{why}");
            assert!(why.contains(std::env::consts::OS), "{why}");
        }
        other => panic!("a collector that reads nothing is not {other:?}"),
    }
}

#[test]
fn asking_it_for_a_reading_is_a_failure_and_never_an_empty_host() {
    let refused = Unported::new("network", "no reader here")
        .collect()
        .expect_err("an empty snapshot would say this host listens on nothing");

    assert!(refused.to_string().contains("network"), "{refused}");
}
