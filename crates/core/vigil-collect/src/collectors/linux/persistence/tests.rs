use std::path::Path;

use super::preload::read_preload;
use super::scripts::describe_script;
use super::{PRELOAD, PersistenceCollector};
use crate::Collector;
use crate::parsers::ScriptFamily;

#[test]
fn a_shell_profile_under_a_directory_the_unit_keeps_private_is_read_as_not_shown() {
    let script = describe_script(Path::new("/tmp/.hidden/.bashrc"), ScriptFamily::Profile);

    assert_eq!(script.present, None);
    assert!(!script.shown);
    assert_eq!(script.readable, None);
    assert_eq!(script.digest, None);
}

#[test]
fn a_shell_profile_the_agent_can_look_at_answers_whether_it_is_there() {
    let here = describe_script(Path::new("/etc/passwd"), ScriptFamily::Profile);
    let nowhere = describe_script(
        Path::new("/etc/there-is-no-such-file"),
        ScriptFamily::Profile,
    );

    assert_eq!(here.present, Some(true));
    assert!(here.shown);
    assert_eq!(nowhere.present, Some(false));
    assert!(nowhere.shown);
}

#[test]
fn a_file_the_agent_could_not_even_look_at_is_not_a_file_that_is_gone() {
    let too_long = "/".to_string() + &"there-is-no-such-directory/".repeat(600) + "profile";

    let refused = describe_script(Path::new(&too_long), ScriptFamily::Profile);

    assert!(refused.shown, "the path is not one the unit keeps private");
    assert_eq!(
        refused.present, None,
        "'we could not look' and 'it is not there' must not be the same answer"
    );
    assert_eq!(refused.readable, Some(false));
}

#[test]
fn reads_this_host_and_names_itself_in_the_snapshot() {
    let collector = PersistenceCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

    let snapshot = collector.collect().expect("something is always readable");

    assert_eq!(snapshot.source, "persistence");
    assert_eq!(snapshot.taken_at, "2026-09-09T12:00:00.000Z");
    assert!(
        snapshot.items.contains_key("preload|/etc/ld.so.preload"),
        "the preload file is an item whether or not it exists"
    );
    for key in snapshot.items.keys() {
        assert!(key.contains('|'), "key shape: {key}");
    }
}

#[test]
fn a_preload_file_that_is_not_there_is_recorded_as_not_being_there() {
    let preload = read_preload();

    assert_eq!(preload.path, PRELOAD);
    assert!(preload.present || preload.readable);
}
