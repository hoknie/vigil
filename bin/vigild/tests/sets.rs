use std::fs;
use std::path::{Path, PathBuf};

use yaml_serde::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn outside_literals(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut kept = String::with_capacity(text.len());
    let mut at = 0;

    while at < chars.len() {
        let here = chars[at];
        let after_identifier = at > 0 && (chars[at - 1].is_alphanumeric() || chars[at - 1] == '_');

        if let Some((hashes, opening)) = raw_string_at(&chars, at).filter(|_| !after_identifier) {
            let closing: Vec<char> = std::iter::once('"')
                .chain(std::iter::repeat_n('#', hashes))
                .collect();
            at += opening;
            while at < chars.len() && !chars[at..].starts_with(&closing) {
                if chars[at] == '\n' {
                    kept.push('\n');
                }
                at += 1;
            }
            at += closing.len();
        } else if here == '"' {
            at += 1;
            while at < chars.len() && chars[at] != '"' {
                if chars[at] == '\\' {
                    at += 1;
                }
                if chars.get(at) == Some(&'\n') {
                    kept.push('\n');
                }
                at += 1;
            }
            at += 1;
        } else if here == '\'' {
            match (chars.get(at + 1), chars.get(at + 2)) {
                (Some('\\'), _) => {
                    at += 2;
                    while at < chars.len() && chars[at] != '\'' {
                        at += 1;
                    }
                    at += 1;
                }
                (Some(_), Some('\'')) => at += 3,
                _ => {
                    kept.push(here);
                    at += 1;
                }
            }
        } else {
            kept.push(here);
            at += 1;
        }
    }

    kept
}

fn raw_string_at(chars: &[char], at: usize) -> Option<(usize, usize)> {
    let mut cursor = at;
    if chars.get(cursor) == Some(&'b') {
        cursor += 1;
    }
    if chars.get(cursor) != Some(&'r') {
        return None;
    }
    cursor += 1;
    let mut hashes = 0;
    while chars.get(cursor) == Some(&'#') {
        hashes += 1;
        cursor += 1;
    }
    (chars.get(cursor) == Some(&'"')).then_some((hashes, cursor + 1 - at))
}

fn ignore_attributes(text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let originals: Vec<&str> = text.lines().collect();
    for (index, line) in outside_literals(text).lines().enumerate() {
        if line.trim().starts_with("#[ignore") {
            let held = originals[index].trim();
            let attribute = match held.find(']') {
                Some(end) => &held[..=end],
                None => held,
            };
            found.push((index + 1, attribute.to_string()));
        }
    }
    found
}

fn heavy_attribute() -> String {
    format!("#[ignore = {}heavy{}]", '"', '"')
}

fn rust_files(under: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(under) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == "target") {
            continue;
        }
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            into.push(path);
        }
    }
}

fn every_rust_file() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for top in ["crates", "bin"] {
        rust_files(&root().join(top), &mut files);
    }
    assert!(files.len() > 100, "only {} file(s) were read", files.len());
    files
}

fn shown(path: &Path) -> String {
    path.strip_prefix(root())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn not_heavy_and_not_said(text: &str) -> Vec<usize> {
    ignore_attributes(text)
        .into_iter()
        .filter(|(_, attribute)| *attribute != heavy_attribute())
        .map(|(line, _)| line)
        .collect()
}

fn reads_the_samples_and_is_heavy(text: &str) -> bool {
    !ignore_attributes(text).is_empty() && outside_literals(text).contains("Golden")
}

#[test]
fn the_reader_finds_an_ignore_attribute_and_nothing_inside_a_literal() {
    assert!(ignore_attributes("#[test]\nfn quick() {}\n").is_empty());
    assert!(ignore_attributes("let text = \"#[ignore]\";\n").is_empty());
    assert!(ignore_attributes("let text = r#\"\n#[ignore]\n\"#;\n").is_empty());

    assert_eq!(
        ignore_attributes("#[test]\n    #[ignore]\nfn slow() {}\n"),
        vec![(2, "#[ignore]".to_string())]
    );
    assert_eq!(
        ignore_attributes("#[ignore = \"slow\"]\nfn slow() {}\n").len(),
        1
    );
}

#[test]
fn every_ignored_test_is_heavy_and_says_so() {
    assert_eq!(
        not_heavy_and_not_said("#[test]\n#[ignore]\nfn a() {}\n"),
        vec![2]
    );
    assert_eq!(
        not_heavy_and_not_said("#[test]\n#[ignore = \"slow\"]\nfn a() {}\n"),
        vec![2]
    );
    assert!(not_heavy_and_not_said("#[test]\n#[ignore = \"heavy\"]\nfn a() {}\n").is_empty());

    let mut found: Vec<String> = Vec::new();
    let mut heavy = 0;
    for file in every_rust_file() {
        let text = fs::read_to_string(&file).expect("readable");
        heavy += ignore_attributes(&text).len();
        for line in not_heavy_and_not_said(&text) {
            found.push(format!("{}:{line}", shown(&file)));
        }
    }

    assert!(
        found.is_empty(),
        "a test is left out of the quick set with #[ignore = \"heavy\"] and with nothing else: \
         the reason is the one word the output of a run shows. Found at:\n{}",
        found.join("\n")
    );
    assert!(
        heavy > 0,
        "no test is marked heavy, so the quick set is the whole set and the night has nothing to add"
    );
}

#[test]
fn no_guard_of_the_repository_is_heavy() {
    let mut guards = Vec::new();
    for directory in ["bin/vigild/tests", "crates/core/vigil-model/tests"] {
        rust_files(&root().join(directory), &mut guards);
    }
    assert!(
        guards.len() >= 5,
        "only {} guard file(s) were read",
        guards.len()
    );

    assert!(!ignore_attributes("#[test]\n#[ignore = \"heavy\"]\nfn guard() {}\n").is_empty());

    let found: Vec<String> = guards
        .iter()
        .flat_map(|file| {
            let text = fs::read_to_string(file).expect("readable");
            ignore_attributes(&text)
                .into_iter()
                .map(|(line, _)| format!("{}:{line}", shown(file)))
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(
        found.is_empty(),
        "a guard of the repository runs on every push: a rule held by a test that the quick set \
         skips is a rule held by the night alone. Found at:\n{}",
        found.join("\n")
    );
}

#[test]
fn a_test_that_reads_the_golden_samples_is_never_heavy() {
    assert!(reads_the_samples_and_is_heavy(
        "use vigil_model::Golden;\n#[test]\n#[ignore = \"heavy\"]\nfn a() { Golden::new(); }\n"
    ));
    assert!(!reads_the_samples_and_is_heavy(
        "#[test]\n#[ignore = \"heavy\"]\nfn a() { let name = \"Golden\"; }\n"
    ));
    assert!(!reads_the_samples_and_is_heavy(
        "use vigil_model::Golden;\n#[test]\nfn a() { Golden::new(); }\n"
    ));

    let found: Vec<String> = every_rust_file()
        .into_iter()
        .filter(|file| reads_the_samples_and_is_heavy(&fs::read_to_string(file).expect("readable")))
        .map(|file| shown(&file))
        .collect();

    assert!(
        found.is_empty(),
        "a file that reads or writes the samples holds a heavy test. A sample the quick set does \
         not read can drift from the code for a day before anyone sees it. Found in:\n{}",
        found.join("\n")
    );
}

fn recipe(text: &str, name: &str) -> Vec<String> {
    let mut lines = text.lines();
    let found = lines.any(|line| {
        line.strip_prefix(name)
            .is_some_and(|rest| rest.starts_with([' ', ':']) && rest.contains(':'))
    });
    assert!(found, "no recipe is called {name}");
    lines
        .take_while(|line| line.starts_with([' ', '\t']) || line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn justice(file: &str) -> String {
    fs::read_to_string(root().join("env/justice").join(file)).expect("a recipe file")
}

#[test]
fn the_quick_gate_and_the_full_gate_are_the_recipes_ci_runs() {
    let tests = justice("qa-test.just");
    let quick = recipe(&tests, "quick");
    let all = recipe(&tests, "all");
    let golden = recipe(&tests, "golden");

    assert!(
        quick.iter().all(|line| !line.contains("--include-ignored"))
            && quick.iter().any(|line| line.ends_with("test --workspace")),
        "the quick set is cargo test --workspace and nothing that includes the ignored: {quick:?}"
    );
    assert!(
        all.iter()
            .any(|line| line.ends_with("test --workspace -- --include-ignored")),
        "the full set is every test, the ignored ones included: {all:?}"
    );
    assert!(
        golden
            .iter()
            .any(|line| line.contains("VIGIL_GOLDEN=write")
                && line.ends_with("-- --include-ignored")),
        "the samples are written by a run that leaves nothing out: {golden:?}"
    );

    let gates = recipe(&justice("quality.just"), "qa-gates");
    let joined = gates.join("\n");
    assert!(
        gates
            .iter()
            .any(|line| line.contains("qa-gates: the set is quick or full")),
        "a set that is neither quick nor full is refused by name: {gates:?}"
    );
    assert!(
        joined.contains("qa-test quick") && joined.contains("qa-test all"),
        "the gate runs the quick set by default and the full one on request: {gates:?}"
    );
    assert!(
        joined.contains("= full ]") && joined.find("= full ]") < joined.find("qa-test all"),
        "the full set is behind the word full: {gates:?}"
    );

    let docker = recipe(&justice("docker-gates.just"), "all");
    assert!(
        docker
            .iter()
            .any(|line| line.ends_with("just qa-gates full")),
        "the gate in the container is the full one: {docker:?}"
    );
}

fn workflow(name: &str) -> Value {
    let path = root().join(".github/workflows").join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{name} is missing"));
    yaml_serde::from_str(&text).unwrap_or_else(|error| panic!("{name} is not yaml: {error}"))
}

fn on_of(document: &Value) -> &Value {
    document
        .get("on")
        .or_else(|| document.get(Value::Bool(true)))
        .expect("a workflow says when it runs")
}

fn triggers(document: &Value) -> Vec<String> {
    let on = on_of(document);
    match on {
        Value::Mapping(map) => map
            .keys()
            .filter_map(|key| key.as_str().map(str::to_string))
            .collect(),
        Value::String(single) => vec![single.clone()],
        Value::Sequence(list) => list
            .iter()
            .filter_map(|key| key.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn job<'a>(document: &'a Value, name: &str) -> &'a Value {
    document
        .get("jobs")
        .and_then(|jobs| jobs.get(name))
        .unwrap_or_else(|| panic!("no job is called {name}"))
}

fn calls(document: &Value, job_name: &str) -> (String, Option<String>) {
    let held = job(document, job_name);
    (
        held.get("uses")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        held.get("with")
            .and_then(|with| with.get("set"))
            .and_then(Value::as_str)
            .map(str::to_string),
    )
}

fn needs(document: &Value, job_name: &str) -> Vec<String> {
    match job(document, job_name).get("needs") {
        Some(Value::String(one)) => vec![one.clone()],
        Some(Value::Sequence(list)) => list
            .iter()
            .filter_map(|name| name.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

#[test]
fn the_pushes_run_the_quick_set_and_the_night_and_the_tag_run_the_full_one() {
    let ci = workflow("ci.yml");
    let triggered = triggers(&ci);
    for trigger in ["push", "pull_request", "workflow_call"] {
        assert!(
            triggered.iter().any(|name| name == trigger),
            "ci.yml lacks {trigger}"
        );
    }

    let steps = job(&ci, "gate")
        .get("steps")
        .and_then(Value::as_sequence)
        .expect("steps");
    let mut full_only = 0;
    for step in steps {
        let run = step.get("run").and_then(Value::as_str).unwrap_or_default();
        if run.contains("--include-ignored") {
            let condition = step.get("if").and_then(Value::as_str).unwrap_or_default();
            assert!(
                condition.contains("== 'full'"),
                "ci.yml runs the ignored tests on a push: {run}"
            );
            full_only += 1;
        }
    }
    assert_eq!(
        full_only, 1,
        "the gate runs the full set once, under the word full"
    );
    let macos = job(&ci, "macos")
        .get("if")
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        macos.contains("== 'full'"),
        "macOS is checked by the full set only: {macos}"
    );

    let release = workflow("release.yml");
    let mut released = triggers(&release);
    released.sort();
    assert_eq!(
        released,
        vec!["push".to_string(), "workflow_dispatch".to_string()],
        "a release runs on a tag and by hand, never on a branch push or a pull request"
    );
    assert!(
        on_of(&release)["push"].get("branches").is_none(),
        "a release does not run on a branch"
    );
    assert_eq!(
        calls(&release, "gate"),
        (
            "./.github/workflows/ci.yml".to_string(),
            Some("full".to_string())
        )
    );
    assert!(needs(&release, "packages").contains(&"gate".to_string()));
    assert!(needs(&release, "publish").contains(&"packages".to_string()));

    let night = workflow("nightly.yml");
    let nightly = triggers(&night);
    assert!(nightly.iter().any(|name| name == "schedule"));
    assert!(nightly.iter().any(|name| name == "workflow_dispatch"));
    assert!(
        !nightly
            .iter()
            .any(|name| name == "push" || name == "pull_request"),
        "the night is not a push"
    );
    assert_eq!(
        calls(&night, "gate"),
        (
            "./.github/workflows/ci.yml".to_string(),
            Some("full".to_string())
        )
    );
    assert_eq!(
        calls(&night, "packages").0,
        "./.github/workflows/packages.yml".to_string()
    );
    for name in ["gate", "packages", "containers", "cost"] {
        assert!(
            needs(&night, name).contains(&"changes".to_string()),
            "{name} does not wait for the decision about code"
        );
        let condition = job(&night, name)
            .get("if")
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert!(
            condition.contains("outputs.code == 'true'"),
            "{name} runs on a night without code: {condition}"
        );
    }

    let packages = triggers(&workflow("packages.yml"));
    assert_eq!(packages, vec!["workflow_call".to_string()]);
}
