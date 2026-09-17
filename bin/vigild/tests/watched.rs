use std::fs;
use std::path::PathBuf;

use vigil_config::{Edit, Watch, put, stop};
use vigil_files::{Files, Watched, Watching};
use vigil_module::{Module, Settings};

const SHIPPED: &str = "\
state_dir: /var/lib/vigil
socket_path: /run/vigil/vigil.sock

files:
  paths:
    - \"/etc/hosts\"
  ceiling_bytes: 1048576

reporters: []
suppressions: []
";

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-16T12:00:00.000Z".to_string()
}

fn a_file(text: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "vigild-watched-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&directory).expect("a directory to write in");
    let path = directory.join("vigil.yaml");
    fs::write(&path, text).expect("writes");
    path
}

fn changed(edit: Edit) -> String {
    match edit {
        Edit::Changed { text, .. } => text,
        other => panic!("{other:?}"),
    }
}

fn as_the_daemon_reads_it(text: &str) -> Watching {
    let path = a_file(text);
    let config = vigild::load(path.to_str().expect("utf-8")).expect("the daemon reads it");
    let settings = Settings::of(at_noon, "files", config.of_the_module("files"));

    Files.check(&settings).expect("the daemon accepts it");
    settings.read().expect("the block parses")
}

#[test]
fn the_paths_this_console_writes_into_the_file_are_the_paths_the_daemon_reads_out_of_it() {
    let added = changed(put(SHIPPED, &Watch::of("/etc/sudoers", None)));
    let held = changed(put(
        &added,
        &Watch::of("/etc/ssl/certs/ca-certificates.crt", Some(8_388_608)),
    ));

    let watching = as_the_daemon_reads_it(&held);

    assert_eq!(
        watching.paths,
        vec![
            Watched::of("/etc/hosts", None),
            Watched::of("/etc/sudoers", None),
            Watched::of("/etc/ssl/certs/ca-certificates.crt", Some(8_388_608)),
        ],
        "the console and the daemon read one file, and an entry one of them writes that the \
         other does not read back is a path an operator believes is watched and is not"
    );
    assert_eq!(
        watching.hashed(),
        vec![
            ("/etc/hosts".to_string(), 1_048_576),
            ("/etc/sudoers".to_string(), 1_048_576),
            ("/etc/ssl/certs/ca-certificates.crt".to_string(), 8_388_608),
        ]
    );
}

#[test]
fn a_path_this_console_takes_out_of_the_file_is_one_the_daemon_no_longer_reads() {
    let added = changed(put(SHIPPED, &Watch::of("/etc/sudoers", None)));
    let after = changed(stop(&added, "/etc/hosts"));

    let watching = as_the_daemon_reads_it(&after);

    assert_eq!(watching.paths, vec![Watched::of("/etc/sudoers", None)]);
    assert_eq!(watching.ceiling_bytes, 1_048_576);
}

#[test]
fn a_file_with_no_files_block_at_all_gains_one_the_daemon_loads_without_complaint() {
    let after = changed(put(
        "state_dir: /var/lib/vigil\nreporters: []\n",
        &Watch::of("/etc/sudoers", Some(4096)),
    ));

    let watching = as_the_daemon_reads_it(&after);

    assert_eq!(
        watching.paths,
        vec![Watched::of("/etc/sudoers", Some(4096))]
    );
}

#[test]
fn what_the_agent_reads_out_of_the_file_is_what_that_module_would_hash_on_the_next_pass() {
    let after = changed(put(SHIPPED, &Watch::of("/etc/hosts", Some(2048))));

    let watching = as_the_daemon_reads_it(&after);

    assert_eq!(
        watching.hashed(),
        vec![("/etc/hosts".to_string(), 2048)],
        "a ceiling written beside one path is the ceiling that path is hashed to, and every \
         other path stays on the one the block names"
    );
}
