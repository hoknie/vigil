#![cfg(target_os = "macos")]

use std::path::{Path, PathBuf};
use std::process::Command;

fn home() -> PathBuf {
    let home = std::env::temp_dir().join(format!("vigil-account-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).expect("a home");
    home
}

fn configure(home: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vigild"))
        .arg("configure")
        .env("HOME", home)
        .output()
        .expect("vigild runs")
}

fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .permissions()
        .mode()
        & 0o777
}

#[test]
fn configure_without_a_home_names_the_file_to_write_and_writes_nothing() {
    if rustix::process::geteuid().is_root() {
        eprintln!("skipped: this test runs as an ordinary account");
        return;
    }
    let directory = home();

    let output = Command::new(env!("CARGO_BIN_EXE_vigild"))
        .arg("configure")
        .env_remove("HOME")
        .current_dir(&directory)
        .output()
        .expect("vigild runs");
    let said = String::from_utf8_lossy(&output.stderr).into_owned();

    assert_eq!(output.status.code(), Some(1), "{said}");
    assert!(
        said.contains(
            "the home directory of this account is not known (HOME): name the configuration file to write"
        ),
        "{said}"
    );
    assert!(!said.contains("asking each collector"), "{said}");
    assert_eq!(std::fs::read_dir(&directory).expect("readable").count(), 0);

    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn an_account_configures_its_own_agent_without_sudo() {
    if rustix::process::geteuid().is_root() {
        eprintln!("skipped: this test runs as an ordinary account");
        return;
    }
    let home = home();

    let first = configure(&home);
    let said = String::from_utf8_lossy(&first.stderr).into_owned();

    let configuration = home.join(".config/vigil/vigil.yaml");
    let job = home.join("Library/LaunchAgents/vigil.vigild.plist");
    assert!(first.status.success(), "{said}");
    assert!(configuration.is_file(), "{said}");
    assert!(home.join(".config/vigil/collectors").is_dir(), "{said}");
    assert!(home.join(".config/vigil/watch_fs.yaml").is_file(), "{said}");
    assert!(job.is_file(), "{said}");
    assert_eq!(mode(&configuration), 0o600);
    assert_eq!(mode(&job), 0o600);
    assert_eq!(mode(&home.join(".config/vigil")), 0o700);
    assert_eq!(mode(&home.join("Library/Logs/vigil")), 0o700);
    assert!(said.contains("launchctl bootstrap gui/"), "{said}");
    assert!(!said.contains("sudo launchctl"), "{said}");

    let written = std::fs::read_to_string(&configuration).expect("readable");
    assert!(
        written.contains(&format!("{}/.local/state/vigil", home.display())),
        "{written}"
    );
    let plist = std::fs::read_to_string(&job).expect("readable");
    assert!(
        plist.contains(&format!("{}/Library/Logs/vigil/vigild.log", home.display())),
        "{plist}"
    );

    let second = configure(&home);
    let again = String::from_utf8_lossy(&second.stderr).into_owned();
    assert!(second.status.success(), "{again}");
    assert!(again.contains("unchanged"), "{again}");
    assert!(!again.contains("wrote"), "{again}");

    let _ = std::fs::remove_dir_all(&home);
}
