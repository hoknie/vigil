use std::path::{Path, PathBuf};

use vigil_collect::Health;
use vigil_config::{Account, Homebrew, Installation, Layout, NO_HOME};

use super::Surveyed;
use super::layout::named;
use super::progress;
use super::run::{Options, configure_with, left_job, next_steps};

fn home(name: &str) -> PathBuf {
    static GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let home = std::env::temp_dir().join(format!(
        "vigil-account-{name}-{}-{}",
        std::process::id(),
        GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&home).expect("a home");
    home
}

fn account(home: &Path) -> Layout {
    Layout::for_account(
        Installation::MACOS,
        &Account {
            uid: 501,
            home: Some(home.to_path_buf()),
        },
    )
    .expect("a layout")
}

fn under_homebrew(home: &Path) -> Layout {
    account(home).under(Some(&Homebrew {
        prefix: "/opt/homebrew".into(),
    }))
}

fn survey() -> Vec<Surveyed> {
    crate::modules::names()
        .into_iter()
        .map(|name| Surveyed {
            name: name.to_string(),
            health: Health::Ok,
        })
        .collect()
}

fn options(layout: &Layout, dry_run: bool, force: bool) -> Options {
    Options {
        path: layout.configuration.clone(),
        force,
        dry_run,
    }
}

#[test]
fn without_a_home_the_refusal_says_so_and_asks_for_a_file_to_write() {
    let said = named("").expect_err("refused");

    assert_eq!(
        said,
        format!("{NO_HOME}: name the configuration file to write")
    );
    assert_eq!(
        said,
        "the home directory of this account is not known (HOME): name the configuration file to write"
    );
}

#[test]
fn an_empty_path_is_refused_and_nothing_is_written() {
    let layout = account(&home("nohome"));
    let nothing = Options {
        path: String::new(),
        force: false,
        dry_run: false,
    };

    let said = configure_with(&layout, &nothing, &survey(), None).expect_err("refused");

    assert!(
        said.ends_with("name the configuration file to write"),
        "{said}"
    );
    assert!(!Path::new(&layout.configuration).exists());
}

#[test]
fn dry_run_prints_the_job_and_writes_nothing() {
    let home = home("dry");
    let layout = account(&home);

    configure_with(&layout, &options(&layout, true, false), &survey(), None).expect("prints");

    assert_eq!(std::fs::read_dir(&home).expect("reads").count(), 0);
    assert!(
        !home
            .join("Library/LaunchAgents/vigil.vigild.plist")
            .exists()
    );
    assert!(!home.join("Library/Logs/vigil").exists());
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn a_job_of_an_account_already_there_is_left_alone_without_force() {
    let home = home("kept");
    let layout = account(&home);
    let job = home.join("Library/LaunchAgents/vigil.vigild.plist");
    std::fs::create_dir_all(job.parent().expect("a parent")).expect("a directory");
    std::fs::write(&job, "mine").expect("a job");

    let said = configure_with(&layout, &options(&layout, false, false), &survey(), None)
        .expect_err("a file was not touched");

    assert!(said.contains("not touched"), "{said}");
    assert_eq!(std::fs::read_to_string(&job).expect("reads"), "mine");
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn under_homebrew_configure_writes_no_launch_agent_and_says_brew_services_start() {
    let home = home("brew");
    let layout = under_homebrew(&home);

    let said =
        configure_with(&layout, &options(&layout, false, false), &survey(), None).expect("writes");

    assert!(said.contains("brew services start vigil"), "{said}");
    assert!(said.contains("vigil ui"), "{said}");
    assert!(!said.contains("launchctl"), "{said}");
    assert!(home.join(".config/vigil/vigil.yaml").exists());
    assert!(!home.join("Library/LaunchAgents").exists());
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn under_homebrew_a_launch_agent_left_by_the_archive_is_named_and_kept() {
    let home = home("left");
    let layout = under_homebrew(&home);
    let job = home.join("Library/LaunchAgents/vigil.vigild.plist");
    std::fs::create_dir_all(job.parent().expect("a parent")).expect("a directory");
    std::fs::write(&job, "mine").expect("a job");

    configure_with(
        &layout,
        &options(&layout, false, false),
        &survey(),
        Some(&job),
    )
    .expect("writes");

    assert_eq!(std::fs::read_to_string(&job).expect("reads"), "mine");
    let lines = left_job(&job);
    assert!(lines.join(" ").contains("vigil.vigild.plist"));
    assert!(
        lines
            .join(" ")
            .contains("launchctl bootout gui/$(id -u)/vigil.vigild"),
        "{lines:?}"
    );
    for line in lines {
        assert!(line.chars().count() <= progress::WIDTH, "{line}");
    }
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn outside_homebrew_the_account_still_gets_its_launch_agent() {
    let home = home("archive");
    let layout = account(&home);

    let said =
        configure_with(&layout, &options(&layout, false, false), &survey(), None).expect("writes");

    assert!(said.contains("launchctl bootstrap gui/501"), "{said}");
    assert!(
        home.join("Library/LaunchAgents/vigil.vigild.plist")
            .exists()
    );
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn every_line_configure_says_under_homebrew_fits_eighty_columns() {
    let name = "a".repeat(32);
    let home = PathBuf::from(format!("/Users/{name}"));
    let layout = under_homebrew(&home);
    let configuration = format!("{}/.config/vigil/vigil.yaml", home.display());

    let said = next_steps(
        &layout,
        Path::new(&configuration),
        Some(&home.display().to_string()),
    );
    let job = home.join("Library/LaunchAgents/vigil.vigild.plist");

    for line in said.lines().map(str::to_string).chain(left_job(&job)) {
        assert!(line.chars().count() <= progress::WIDTH, "{line}");
    }
}
