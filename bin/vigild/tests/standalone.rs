use std::path::{Path, PathBuf};
use std::process::Command;

const FOREIGN_PREFIXES: [&str; 2] = ["waf-", "waf_"];

#[test]
fn nothing_in_the_dependency_graph_belongs_to_another_product() {
    let graph = resolved_graph();

    let foreign: Vec<&str> = graph
        .iter()
        .map(String::as_str)
        .filter(|name| {
            FOREIGN_PREFIXES
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
        .collect();

    assert!(
        foreign.is_empty(),
        "vigil links {foreign:?}. The contract with is-waf is versioned data and the arrow \
         points one way: vigil never links waf-*. A transitive edge counts."
    );
}

#[test]
fn the_graph_is_read_whole_so_a_transitive_edge_cannot_hide_in_it() {
    let graph = resolved_graph();

    for ours in [
        "vigil-model",
        "vigil-rules",
        "vigil-collect",
        "vigil-store",
        "vigil-report",
        "vigild",
    ] {
        assert!(
            graph.iter().any(|name| name == ours),
            "{ours} is not in the graph this guard read, so the guard is reading the wrong thing"
        );
    }
    assert!(
        graph.iter().any(|name| name == "serde"),
        "no third-party crate is in the graph: cargo metadata returned only the workspace"
    );
}

#[test]
fn the_console_links_no_collector_and_no_rule_not_even_to_test_itself() {
    let forbidden = [
        "vigil-collect",
        "vigil_collect",
        "vigil-rules",
        "vigil_rules",
    ];

    let linked = what_links("bin/vigil");
    for name in forbidden {
        assert!(
            !linked.contains(&name.to_string()),
            "the console links {name}. Its only source of data is the daemon's socket: a second \
             implementation of a reading here diverges from the one the rules use, and the two \
             disagree during the incident the console was opened for. A dev-dependency counts, \
             because a fixture built from a collector is that second implementation."
        );
    }

    assert!(
        linked
            .iter()
            .any(|name| name == "vigil-model" || name == "vigil_model"),
        "the console links no vigil crate at all, so this guard is reading the wrong node"
    );
}

#[test]
fn the_arrow_between_the_products_is_read_as_an_edge_and_not_as_a_list_of_packages() {
    let daemon = what_links("bin/vigild");

    assert!(
        daemon
            .iter()
            .any(|name| name.starts_with("vigil-collect") || name.starts_with("vigil_collect")),
        "the daemon does not link the collectors, so what_links is not reading dependency edges"
    );
}

#[test]
fn a_panicking_collector_switches_itself_off_rather_than_killing_the_daemon() {
    assert!(
        profile("release").contains("panic = \"unwind\""),
        "[profile.release] no longer unwinds. A collector that panics must be caught and \
         reported as a failed reading; under panic = \"abort\" one bad parse stops the agent \
         watching the host."
    );
}

#[test]
fn a_rebuild_pays_for_line_tables_and_not_for_a_full_debug_map() {
    assert!(
        profile("dev").contains("debug = \"line-tables-only\""),
        "[profile.dev] asks for full debug information again. The lever on rebuild time here \
         is the volume of debug info, not the number of dependencies: every relink walks it."
    );
}

#[test]
fn the_help_is_written_for_eighty_columns_rather_than_reflowed_at_runtime() {
    let clap = dependency("clap");

    assert!(
        !clap.contains("wrap_help"),
        "clap gained the wrap_help feature. It probes the terminal width to reflow text that \
         is already written for eighty columns, which is the floor this console renders at."
    );
}

fn what_links(manifest_directory: &str) -> Vec<String> {
    let metadata = metadata();
    let id = metadata["packages"]
        .as_array()
        .expect("a list of packages")
        .iter()
        .find(|package| {
            package["manifest_path"]
                .as_str()
                .is_some_and(|path| path.contains(manifest_directory))
        })
        .and_then(|package| package["id"].as_str())
        .unwrap_or_else(|| panic!("{manifest_directory} is not a package of this workspace"))
        .to_string();

    metadata["resolve"]["nodes"]
        .as_array()
        .expect("a resolved graph with edges in it")
        .iter()
        .find(|node| node["id"].as_str() == Some(id.as_str()))
        .expect("the package has a node in the resolved graph")["deps"]
        .as_array()
        .expect("a list of edges")
        .iter()
        .filter_map(|dep| dep["name"].as_str().map(str::to_string))
        .collect()
}

fn metadata() -> serde_json::Value {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--all-features",
            "--manifest-path",
        ])
        .arg(workspace().join("Cargo.toml"))
        .output()
        .expect("cargo metadata runs");

    assert!(
        output.status.success(),
        "cargo metadata: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("cargo metadata prints json")
}

fn resolved_graph() -> Vec<String> {
    metadata()["packages"]
        .as_array()
        .expect("a list of packages")
        .iter()
        .filter_map(|package| package["name"].as_str().map(str::to_string))
        .collect()
}

fn manifest() -> String {
    std::fs::read_to_string(workspace().join("Cargo.toml")).expect("the workspace manifest")
}

fn profile(name: &str) -> String {
    section(&manifest(), &format!("[profile.{name}]"))
}

fn dependency(name: &str) -> String {
    let table = section(&manifest(), "[workspace.dependencies]");
    table
        .lines()
        .find(|line| line.starts_with(&format!("{name} =")))
        .unwrap_or_else(|| panic!("{name} is not a workspace dependency"))
        .to_string()
}

fn section(manifest: &str, heading: &str) -> String {
    let body = manifest
        .split(heading)
        .nth(1)
        .unwrap_or_else(|| panic!("the workspace manifest has no {heading}"));

    match body.find("\n[") {
        Some(end) => body[..end].to_string(),
        None => body.to_string(),
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root")
}
