use std::fs;
use std::path::{Path, PathBuf};

const OF_THE_DAEMON: &str = "bin/vigild/src/modules/registry.rs";

const OF_THE_CONSOLE: &str = "bin/vigil/src/ui/sections.rs";

#[test]
fn the_two_binaries_watch_and_draw_the_same_modules_in_the_same_order() {
    let daemon = listed(OF_THE_DAEMON);
    let console = listed(OF_THE_CONSOLE);

    assert_eq!(
        daemon, console,
        "each binary composes its own list of modules, and a module in one of them and not \
         in the other is a reading nobody draws or a screen nothing fills. The order is the \
         order the readings are taken in and the order the sections are numbered in, so it \
         is the same order in both."
    );
}

#[test]
fn every_module_of_the_workspace_is_in_that_list() {
    let crates = workspace().join("crates/collectors");
    let mut shipped: Vec<String> = fs::read_dir(&crates)
        .unwrap_or_else(|error| panic!("{}: {error}", crates.display()))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .map(|named| named.replace('-', "_"))
        .collect();
    shipped.sort();

    let mut listed: Vec<String> = listed(OF_THE_DAEMON)
        .into_iter()
        .map(|(crate_name, _)| crate_name)
        .collect();
    listed.sort();

    assert_eq!(
        shipped, listed,
        "a crate under crates/collectors that no binary lists is a module this build \
         compiles and never asks for a reading"
    );
}

fn listed(path: &str) -> Vec<(String, String)> {
    let at = workspace().join(path);
    let text = fs::read_to_string(&at).unwrap_or_else(|error| panic!("{}: {error}", at.display()));

    let named: Vec<(String, String)> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Box::new(vigil_"))
        .filter_map(|rest| rest.split_once("::"))
        .map(|(crate_name, module)| {
            (
                format!("vigil_{crate_name}"),
                module.trim_end_matches("),").to_string(),
            )
        })
        .collect();

    assert!(
        !named.is_empty(),
        "{path} lists no module, so this guard reads nothing"
    );
    named
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root")
}
