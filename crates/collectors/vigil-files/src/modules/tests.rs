use vigil_module::{Module, Settings};

use super::Files;
use super::files::Watching;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_a_watched_path_walks_to_the_row_keyed_by_that_path() {
    for key in ["file|/etc/ssh/sshd_config", "directory|/usr/local/bin"] {
        assert_eq!(
            Files.row_of(key),
            Some(key.to_string()),
            "this collector keys its rows the way its rules key their findings, and cutting \
             the first segment would look for a row that was never written"
        );
    }
    assert!(!Files.raised("resource|disk|/var"));
    assert_eq!(Files.row_of("resource|disk|/var"), None);
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Files.name(), "files");
    assert_eq!(Files.every_seconds(), 300);
    assert_eq!(Files.unit(), None);
    assert!(!Files.rules(&Settings::plain(at_noon)).is_empty());
    assert!(Files.section().is_some());
}

#[test]
fn a_host_whose_file_names_no_path_watches_the_list_this_product_ships() {
    let watching = Watching::default();

    assert!(watching.paths.contains(&"/etc/ssh/sshd_config".to_string()));
    assert_eq!(watching.ceiling_bytes, 1024 * 1024);
    for path in &watching.paths {
        assert!(
            !path.ends_with('/'),
            "{path} is a directory, and a directory in this list is a walk that hashes \
             everything under it: the shortest path to an agent the operator turns off"
        );
    }
}

#[test]
fn a_host_that_names_its_own_paths_watches_those_and_not_the_shipped_ones() {
    let named = Settings::of(
        at_noon,
        "files",
        serde_json::json!({ "paths": ["/etc/sudoers"] }),
    );
    let watching: Watching = named.read().expect("the sample parses");

    assert_eq!(watching.paths, vec!["/etc/sudoers".to_string()]);
    assert_eq!(
        watching.ceiling_bytes,
        1024 * 1024,
        "a file that names the paths leaves the ceiling on the one this module ships"
    );
}
