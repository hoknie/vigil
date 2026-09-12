use serde::Deserialize;
use serde_json::json;
use vigil_model::Rfc3339;

use super::*;

fn at_noon() -> Rfc3339 {
    "2026-09-12T12:00:00.000Z".to_string()
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
struct Watched {
    paths: Vec<String>,
    ceiling_bytes: u64,
}

#[test]
fn a_module_the_configuration_file_says_nothing_about_gets_its_own_defaults() {
    let settings = Settings::plain(at_noon);

    let watched: Watched = settings.read().expect("silence is the default");

    assert_eq!(watched, Watched::default());
}

#[test]
fn what_the_file_says_about_a_module_reaches_that_module_and_nothing_else() {
    let settings = Settings::of(
        at_noon,
        "files",
        json!({ "paths": ["/etc/passwd"], "ceiling_bytes": 4096 }),
    );

    let watched: Watched = settings.read().expect("readable");

    assert_eq!(watched.paths, vec!["/etc/passwd".to_string()]);
    assert_eq!(watched.ceiling_bytes, 4096);
}

#[test]
fn a_misspelled_key_names_the_module_it_was_written_under_rather_than_a_line_number() {
    let settings = Settings::of(at_noon, "files", json!({ "path": ["/etc/passwd"] }));

    let refusal = settings
        .read::<Watched>()
        .expect_err("a key nobody declares");

    assert_eq!(refusal.key, "files");
    assert!(
        refusal.why.contains("path"),
        "an operator who typed the key has to be told which word was wrong: {}",
        refusal.why
    );
}

#[test]
fn the_clock_a_module_reads_the_host_with_is_handed_to_it_and_never_taken_from_the_host() {
    let settings = Settings::plain(at_noon);

    assert_eq!((settings.now())(), "2026-09-12T12:00:00.000Z");
}
