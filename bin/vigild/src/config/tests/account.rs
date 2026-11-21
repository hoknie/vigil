use std::path::PathBuf;

use vigil_config::{Account, Installation, Layout, NO_HOME};

use crate::config::load::load_for;

fn file(name: &str, text: &str) -> String {
    let dir = std::env::temp_dir().join(format!("vigil-config-account-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(name);
    std::fs::write(&path, text).expect("write");
    path.to_str().expect("utf-8").to_string()
}

fn ann() -> Result<Layout, String> {
    Layout::for_account(
        Installation::MACOS,
        &Account {
            uid: 501,
            home: Some(PathBuf::from("/Users/ann")),
        },
    )
}

fn homeless() -> Result<Layout, String> {
    Layout::for_account(
        Installation::MACOS,
        &Account {
            uid: 501,
            home: None,
        },
    )
}

#[test]
fn an_empty_file_on_a_mac_of_an_account_keeps_the_state_under_its_home() {
    let config = load_for(&file("empty.yaml", "{}\n"), &ann()).expect("parses");

    assert_eq!(config.state_dir, "/Users/ann/.local/state/vigil");
    assert_eq!(
        config.socket_path,
        "/Users/ann/.local/state/vigil/vigil.sock"
    );
}

#[test]
fn a_written_state_dir_wins_over_the_account() {
    let config = load_for(&file("state.yaml", "state_dir: /srv/state\n"), &ann()).expect("parses");

    assert_eq!(config.state_dir, "/srv/state");
    assert_eq!(
        config.socket_path,
        "/Users/ann/.local/state/vigil/vigil.sock"
    );
}

#[test]
fn a_written_socket_path_wins_over_the_account() {
    let config =
        load_for(&file("socket.yaml", "socket_path: /tmp/v.sock\n"), &ann()).expect("parses");

    assert_eq!(config.socket_path, "/tmp/v.sock");
    assert_eq!(config.state_dir, "/Users/ann/.local/state/vigil");
}

#[test]
fn an_account_without_a_home_and_without_places_does_not_start() {
    let error = load_for(&file("homeless.yaml", "{}\n"), &homeless()).expect_err("refused");

    assert!(error.cause.contains(NO_HOME), "{error}");
    assert!(error.cause.contains("state_dir"), "{error}");
}

#[test]
fn an_account_without_a_home_that_names_both_places_starts() {
    let text = "state_dir: /srv/state\nsocket_path: /srv/state/v.sock\n";

    let config = load_for(&file("homeless-named.yaml", text), &homeless()).expect("starts");

    assert_eq!(config.state_dir, "/srv/state");
}

#[test]
fn on_linux_an_empty_file_keeps_var_lib_whoever_runs_it() {
    let layout = Layout::for_account(
        Installation::LINUX,
        &Account {
            uid: 501,
            home: Some(PathBuf::from("/home/ann")),
        },
    );

    let config = load_for(&file("linux.yaml", "{}\n"), &layout).expect("parses");

    assert_eq!(config.state_dir, "/var/lib/vigil");
    assert_eq!(config.socket_path, "/run/vigil/vigil.sock");
}
