use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "env/scripts/audit-check.sh";

const REACHES_EVERY_PROCESS_ON_THE_MACHINE: [&str; 4] = ["pkill", "killall", "pgrep", "pidof"];

fn recipe() -> String {
    let path = workspace().join(RECIPE);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn instructions(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .collect()
}

#[test]
fn the_only_audit_daemon_this_recipe_may_stop_is_the_one_it_started() {
    let text = recipe();

    for reaching in REACHES_EVERY_PROCESS_ON_THE_MACHINE {
        for line in instructions(&text) {
            assert!(
                !line.contains(reaching),
                "the recipe runs {reaching:?}: {line:?}. This container shares the host's \
                 process table, so a command that selects processes by name selects every \
                 auditd on the machine — including one that is starting up and has not \
                 registered yet, which the refusal at the top cannot see. The daemon this \
                 recipe may stop is the pid the kernel registered after it started its own, \
                 and that pid only"
            );
        }
    }

    assert!(
        text.contains("kill -TERM \"$auditor\""),
        "the one daemon it stops is named by the variable holding that pid"
    );
    assert!(
        text.contains("auditor=\"$started\""),
        "and that variable is set from the pid the kernel registered, not from a search"
    );
    assert!(
        text.contains("[ \"$started\" = \"$theirs\" ]"),
        "with a check that the registered pid is not the one that was already there: a \
         daemon we did not start stays running, whatever state its registration is in"
    );
}

#[test]
fn every_rule_it_loads_it_takes_out_again_by_the_specification_it_put_in() {
    let text = recipe();

    assert!(
        text.contains("auditctl -d \"$ACTION\" \"${RULE[@]}\""),
        "the rule is deleted by the same specification it was added with"
    );
    for line in instructions(&text) {
        assert!(
            !line.contains("auditctl -D"),
            "the recipe runs `auditctl -D`, which deletes every audit rule in the kernel and \
             not only the one it added: {line:?}"
        );
    }
}

#[test]
fn it_says_what_it_touches_before_it_touches_it() {
    let text = recipe();

    assert!(
        text.contains("KERNEL"),
        "audit has no namespace, so an operator has to be told before anything happens that \
         this is the host's kernel and not the container's"
    );
    let warning = text
        .find("audit has no namespace")
        .expect("the sentence that says why");
    let first_change = text
        .find("auditctl -e 1")
        .expect("the first thing it changes");
    assert!(
        warning < first_change,
        "the warning is printed after the first change to the kernel"
    );
}

#[test]
fn the_recipe_is_not_part_of_the_gate_because_the_gate_may_not_touch_the_machine_it_runs_on() {
    let quality = fs::read_to_string(workspace().join("env/justice/quality.just"))
        .expect("the quality recipes");
    let docker = fs::read_to_string(workspace().join("env/justice/docker.just"))
        .expect("the docker recipes");

    assert!(
        !quality.contains("audit-check") && !quality.contains("docker-audit"),
        "`just check` is the gate, and a gate that loads an audit rule into the kernel of \
         whatever machine ran it is a gate nobody can run twice at once"
    );
    assert!(
        docker.contains("docker-audit"),
        "the live audit path is a recipe of its own"
    );
    assert!(
        !docker.contains(
            "docker-check:\n    {{ RUN }} just check\n    {{ COMPOSE }} run --rm --build audit"
        ),
        "and docker-check does not pull it in"
    );
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root")
}
