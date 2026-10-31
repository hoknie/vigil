use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SECTIONS: &[&str] = &[
    "## The decision it follows from",
    "## Dependencies",
    "## Context",
];

fn members() -> Vec<(String, PathBuf)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(&root)
        .output()
        .expect("cargo metadata runs");
    assert!(output.status.success(), "cargo metadata failed");
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata speaks JSON");

    metadata["packages"]
        .as_array()
        .expect("a list of packages")
        .iter()
        .map(|package| {
            let name = package["name"].as_str().expect("a name").to_string();
            let manifest = PathBuf::from(package["manifest_path"].as_str().expect("a manifest"));
            let directory = manifest.parent().expect("a directory").to_path_buf();
            (name, directory)
        })
        .collect()
}

fn missing_from(name: &str, text: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let title = format!("# {name}");
    if text.lines().next().map(str::trim) != Some(title.as_str()) {
        missing.push(format!("the title `{title}` on the first line"));
    }
    for section in SECTIONS {
        if !text.lines().any(|line| line.trim() == *section) {
            missing.push(format!("`{section}`"));
        }
    }
    missing
}

#[test]
fn the_readme_reader_names_what_is_missing() {
    let whole =
        "# vigil-x\n\nWhy.\n\n## The decision it follows from\n\n## Dependencies\n\n## Context\n";
    assert!(missing_from("vigil-x", whole).is_empty());

    let untitled = whole.replacen("# vigil-x", "# something else", 1);
    assert_eq!(missing_from("vigil-x", &untitled).len(), 1);

    let without_context = whole.replace("## Context\n", "");
    assert_eq!(
        missing_from("vigil-x", &without_context),
        vec!["`## Context`".to_string()]
    );
}

#[test]
fn every_member_of_the_workspace_has_a_readme_that_says_why_it_exists() {
    let mut found = Vec::new();
    for (name, directory) in members() {
        let readme = directory.join("README.md");
        match fs::read_to_string(&readme) {
            Err(_) => found.push(format!("{name}: no README.md beside its Cargo.toml")),
            Ok(text) => {
                for missing in missing_from(&name, &text) {
                    found.push(format!("{name}: its README.md lacks {missing}"));
                }
            }
        }
    }

    assert!(
        found.is_empty(),
        "every crate says what it is for, the decision it follows from, what it depends on and \
         where it sits, so a reader of the workspace does not have to work it out from the code:\n{}",
        found.join("\n")
    );
}
