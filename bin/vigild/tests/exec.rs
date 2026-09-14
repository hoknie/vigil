use std::fs;
use std::path::{Path, PathBuf};

const STARTS_A_PROGRAM: [&str; 3] = ["Command::new", "process::Command", "exec("];

const THE_ONE_PLACE: &str = "bin/vigild/src/collector/host.rs";

const THE_OTHER_PLACE: &str = "bin/vigild/src/killing/destruction.rs";

fn sources() -> Vec<(String, String)> {
    let mut read = Vec::new();
    for tree in [
        "bin/vigild/src",
        "bin/vigil/src",
        "bin/vigil-audit-plugin/src",
        "crates",
    ] {
        walk(&workspace().join(tree), &mut read);
    }
    assert!(read.len() > 100, "only {} files were read", read.len());
    read
}

fn walk(at: &Path, into: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, into);
            continue;
        }
        if path.extension().is_some_and(|kind| kind == "rs") {
            let named = path
                .strip_prefix(workspace())
                .unwrap_or(&path)
                .display()
                .to_string();
            into.push((named, fs::read_to_string(&path).expect("a source file")));
        }
    }
}

#[test]
fn the_two_places_that_start_a_program_are_the_operators_command_and_the_kill_it_asked_for() {
    let mut starts: Vec<String> = Vec::new();

    for (named, text) in sources() {
        if STARTS_A_PROGRAM.iter().any(|how| text.contains(how)) {
            starts.push(named);
        }
    }
    starts.sort();

    let mut expected = vec![THE_ONE_PLACE.to_string(), THE_OTHER_PLACE.to_string()];
    expected.sort();

    assert_eq!(
        starts, expected,
        "this daemon watches a host it does not touch, and it starts a program in two \
         places, both of them an operator's own instruction arriving at the front door: \
         `vigild collector <name> enable`, and the kill a person confirmed at the console \
         after switching killing on in vigil.yaml. The second was added on 2026-09-14 with \
         the trade-off stated: see docs/designs/2026-09-14-DESIGN-console-kill.md. A third \
         place is a path from a reading, a rule or an unasked-for message to execution on \
         the host, and that is the failure the contract has no reverse channel for"
    );
}

#[test]
fn the_watching_loop_and_the_rules_reach_neither_of_them() {
    for (named, text) in sources() {
        let watches = named.starts_with("bin/vigild/src/loops")
            || named.starts_with("crates/core/vigil-rules")
            || named.starts_with("bin/vigild/src/collector/edit.rs");
        if !watches {
            continue;
        }
        for how in ["killing::carry_out", "destruction::", "host::enable"] {
            assert!(
                !text.contains(how),
                "{named} reaches something that acts on this host. A reading that ends in a \
                 kill is a rule that kills, and nobody asked for that"
            );
        }
    }
}

#[test]
fn the_place_that_starts_one_is_reached_from_the_command_line_and_from_nowhere_else() {
    let text = fs::read_to_string(workspace().join(THE_ONE_PLACE)).expect("the one place");

    assert!(
        text.contains("/usr/bin/systemctl"),
        "the one program it starts is systemctl, by its absolute path"
    );

    let callers: Vec<String> = sources()
        .into_iter()
        .filter(|(named, text)| {
            named != THE_ONE_PLACE
                && (text.contains("host::enable")
                    || text.contains("host::disable")
                    || text.contains("host::standing"))
        })
        .map(|(named, _)| named)
        .collect();

    assert_eq!(
        callers,
        vec!["bin/vigild/src/collector/run.rs".to_string()],
        "only the operator's command reaches it; the watching loop, the socket and the rules \
         do not"
    );

    let loops = fs::read_to_string(workspace().join("bin/vigild/src/loops/cycle.rs"))
        .expect("the watching loop");
    let socket = fs::read_to_string(workspace().join("bin/vigild/src/socket/answer.rs"))
        .expect("the socket");
    for reached in [loops, socket] {
        assert!(!reached.contains("collector::"), "{reached}");
    }
}

#[test]
fn the_collectors_themselves_start_nothing_whatever_they_have_to_read() {
    for (named, text) in sources() {
        if !named.starts_with("crates/core/vigil-collect") {
            continue;
        }
        for how in STARTS_A_PROGRAM {
            assert!(
                !text.contains(how),
                "{named} starts a program. A collector reads files and /proc; the day one \
                 runs a binary is the day `nft` runs inside a daemon with capabilities, which \
                 is the path this product chose the packaging over"
            );
        }
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root")
}
