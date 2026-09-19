use std::path::{Path, PathBuf};

use vigil_collect::Health;
use vigil_config::{Installation, Switched, blocks_in, switched};

use super::Surveyed;
use super::documents::{documents, names};
use super::plan::{Planned, plan};
use super::progress::{self, Action, WIDTH};
use super::run::{Options, configure_with};
use super::shipped::{COLLECTORS, CONFIGURATION, WATCH_LIST};
use crate::Config;

fn surveyed(name: &str, health: Health) -> Surveyed {
    Surveyed {
        name: name.to_string(),
        health,
    }
}

fn every_one_runs() -> Vec<Surveyed> {
    crate::modules::names()
        .into_iter()
        .map(|name| surveyed(name, Health::Ok))
        .collect()
}

fn survey() -> Vec<Surveyed> {
    vec![
        surveyed("network", Health::Ok),
        surveyed("users", Health::Ok),
        surveyed(
            "launches",
            Health::Unavailable(
                "auditd is not running — program launches are not visible (there is no /var/log/audit/audit.log)".into(),
            ),
        ),
    ]
}

fn directory(name: &str) -> PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-configure-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos()
                + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&directory).expect("a directory");
    directory
}

fn options(directory: &Path) -> Options {
    Options {
        path: directory.join("vigil.yaml").display().to_string(),
        force: false,
        dry_run: false,
    }
}

fn loaded(directory: &Path) -> Config {
    crate::config::load(directory.join("vigil.yaml").to_str().expect("utf-8"))
        .expect("the daemon reads what configure wrote")
}

fn written(planned: &[Planned], path: &str) -> Option<String> {
    planned
        .iter()
        .find(|file| file.path.ends_with(path))
        .and_then(|file| file.text.clone())
}

fn shipped(path: &str) -> &'static str {
    COLLECTORS
        .iter()
        .find(|shipped| shipped.path == path)
        .map(|shipped| shipped.text)
        .unwrap_or_else(|| panic!("{path} is not shipped"))
}

fn shipped_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config")
}

#[test]
fn what_it_writes_is_a_file_the_daemon_reads() {
    let directory = directory("reads");

    configure_with(&options(&directory), &survey()).expect("writes");
    let config = loaded(&directory);

    assert_eq!(
        config.collectors,
        Some(vec!["network".to_string(), "users".to_string()]),
        "only what can run here is switched on"
    );
    assert_eq!(
        config.apart.collectors_at,
        Some(directory.join("collectors")),
        "a configuration written anywhere but /etc/vigil points at the collectors beside it"
    );
    assert_eq!(config.state_dir, Config::default().state_dir);
    assert_eq!(config.socket_path, Config::default().socket_path);
    assert_eq!(config.retention_days, Config::default().retention_days);
    assert!(config.suppressions.is_empty());
    assert!(config.reporters.is_empty());
    assert_eq!(
        config.apart.suppressions_at,
        Some(directory.join("suppressions")),
        "what the console silences lands in a directory this command never writes, so \
         `vigild configure --force` cannot take it away"
    );
    assert_eq!(config.apart.reporters_at, Some(directory.join("reporters")));
    assert!(!directory.join("suppressions").exists());
    assert!(!directory.join("reporters").exists());
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn what_it_writes_on_a_host_where_everything_runs_is_a_file_the_daemon_reads() {
    let directory = directory("everything");

    configure_with(&options(&directory), &every_one_runs()).expect("writes");
    let config = loaded(&directory);

    assert_eq!(
        config.collectors,
        Some(
            crate::modules::names()
                .into_iter()
                .map(str::to_string)
                .collect()
        )
    );
    assert!(directory.join("watch_fs.yaml").exists());
    let files = std::fs::read_to_string(directory.join("collectors/files.yaml")).expect("a file");
    assert!(
        files.contains(&format!("{}/watch_fs.yaml", directory.display())),
        "the watch list the files collector is pointed at is the one written beside it: {files}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_collector_that_cannot_run_here_gets_no_file_and_the_console_says_why() {
    let planned = plan(Path::new("/etc/vigil/vigil.yaml"), &survey());

    assert_eq!(written(&planned, "collectors/launches.yaml"), None);
    assert!(written(&planned, "collectors/network.yaml").is_some());
    assert!(
        !planned
            .iter()
            .filter_map(|file| file.text.as_deref())
            .any(|text| text.contains("auditd is not running")),
        "the files hold settings; what this host answered is said on the console"
    );

    let said = progress::survey(&survey()).join("\n");
    assert!(
        said.contains("  no    launches  auditd is not running"),
        "{said}"
    );
    assert!(said.contains("  ok    network\n"), "{said}");
}

#[test]
fn a_degraded_collector_is_written_and_the_console_says_what_it_is_missing() {
    let survey = vec![surveyed(
        "launches",
        Health::Degraded("the audit rule is not loaded; run augenrules --load".into()),
    )];

    let planned = plan(Path::new("/etc/vigil/vigil.yaml"), &survey);
    let said = progress::survey(&survey).join("\n");

    assert!(written(&planned, "collectors/launches.yaml").is_some());
    assert!(said.contains("  warn  launches  the audit rule"), "{said}");
}

#[test]
fn a_file_of_two_collectors_is_written_with_the_block_of_the_one_that_runs_here() {
    let survey = vec![
        surveyed("containers", Health::Ok),
        surveyed(
            "containers-engines",
            Health::Unavailable("no engine answered".into()),
        ),
    ];

    let text = written(
        &plan(Path::new("/etc/vigil/vigil.yaml"), &survey),
        "collectors/containers.yaml",
    )
    .expect("containers runs here");

    assert_eq!(names(&text), vec!["containers".to_string()]);
    assert!(!text.contains("---"), "{text}");
}

#[test]
fn every_key_that_lets_the_console_change_this_host_is_written_out_and_switched_off() {
    let planned = plan(Path::new("/etc/vigil/vigil.yaml"), &every_one_runs());
    let text: String = planned
        .iter()
        .filter_map(|file| file.text.clone())
        .collect();

    for (file, key) in [
        ("collectors/processes.yaml", "  killing:\n"),
        ("collectors/users.yaml", "  accounts:\n"),
        ("collectors/persistence.yaml", "  units:\n"),
    ] {
        let written = written(&planned, file).expect("written");
        assert!(written.contains(key), "{key} is not in {file}: {written}");
    }
    assert_eq!(
        text.matches("\n    from_the_console: false\n").count(),
        3,
        "a file written by the wizard switches on nothing this agent does to a host it did \
         not set up, and a key missing from it is a key nobody knows to look for: {text}"
    );
}

#[test]
fn on_a_host_where_everything_runs_it_writes_exactly_the_files_this_product_ships() {
    let here = Installation::here();
    let planned = plan(Path::new(here.configuration), &every_one_runs());
    let shipped = shipped_directory();

    for file in &planned {
        let relative = file
            .path
            .strip_prefix(here.configuration_directory)
            .expect("under the directory this system installs to");
        let on_disk = match relative == Path::new("vigil.yaml") {
            true => shipped.join("vigil.example.yaml"),
            false => shipped.join(relative),
        };
        let written = std::fs::read_to_string(&on_disk)
            .unwrap_or_else(|error| panic!("{}: {error}", on_disk.display()));
        assert_eq!(
            file.text.as_deref(),
            Some(here.moved_from(&Installation::LINUX, &written).as_str()),
            "{} is not what config/ ships, moved to the places of this system",
            file.path.display()
        );
    }

    let mut every: Vec<String> = std::fs::read_dir(shipped.join("collectors"))
        .expect("config/collectors")
        .map(|entry| {
            format!(
                "collectors/{}",
                entry.expect("an entry").file_name().to_string_lossy()
            )
        })
        .collect();
    every.sort();
    let mut known: Vec<String> = COLLECTORS
        .iter()
        .map(|shipped| shipped.path.to_string())
        .collect();
    known.sort();
    assert_eq!(
        every, known,
        "a file under config/collectors that the wizard does not know is a file the package \
         installs and `vigild configure` never writes"
    );
}

#[test]
fn every_collector_this_build_has_is_shipped_in_exactly_one_block() {
    let shipped: Vec<String> = COLLECTORS
        .iter()
        .flat_map(|shipped| names(shipped.text))
        .collect();

    for name in crate::modules::names() {
        assert_eq!(
            shipped.iter().filter(|named| *named == name).count(),
            1,
            "{name}: a collector the shipped files have no block for runs nowhere the package \
             is installed, and two blocks are refused at start-up"
        );
    }
    assert_eq!(shipped.len(), crate::modules::names().len(), "{shipped:?}");
}

#[test]
fn the_shipped_blocks_write_out_the_defaults_and_switch_nothing_on() {
    for shipped in COLLECTORS {
        for block in blocks_in(Path::new(shipped.path), shipped.text).expect("parses") {
            assert!(block.enabled, "{}: {}", shipped.path, block.name);
            assert_eq!(
                block.schedule,
                crate::modules::every_seconds_of(&block.name),
                "{}: the period written is the one {} declares",
                shipped.path,
                block.name
            );
            for verb in ["killing", "accounts", "units"] {
                if let Some(said) = block.settings.get(verb) {
                    assert_eq!(
                        said.get("from_the_console")
                            .and_then(|value| value.as_bool()),
                        Some(false),
                        "{}: {verb}",
                        shipped.path
                    );
                }
            }
        }
    }

    let launches = blocks_in(
        Path::new("launches.yaml"),
        shipped("collectors/launches.yaml"),
    )
    .expect("parses");
    assert_eq!(launches[0].name, "launches");
    assert_eq!(
        launches[0]
            .settings
            .get("record_arguments")
            .and_then(|value| value.as_bool()),
        Some(vigil_launches::Watching::default().record_arguments),
        "the arguments of a command carry its secrets, and the shipped file keeps them only \
         when the collector would"
    );
    let resources = blocks_in(
        Path::new("resources.yaml"),
        shipped("collectors/resources.yaml"),
    )
    .expect("parses");
    assert_eq!(resources[0].name, "resources");
    for (key, default) in [
        ("clock_skew_seconds", vigil_resources::CLOCK_SKEW_SECONDS),
        ("disk_free_percent", vigil_resources::DISK_FREE_PERCENT),
        ("inode_free_percent", vigil_resources::INODE_FREE_PERCENT),
    ] {
        assert_eq!(
            resources[0]
                .settings
                .get(key)
                .and_then(|value| value.as_u64()),
            Some(u64::from(default)),
            "{key}"
        );
    }
}

#[test]
fn every_key_in_the_shipped_files_has_a_comment_above_it() {
    let mut texts: Vec<(&str, &str)> = vec![("vigil.yaml", CONFIGURATION)];
    texts.extend(
        COLLECTORS
            .iter()
            .filter(|shipped| shipped.path != "collectors/files.yaml")
            .map(|shipped| (shipped.path, shipped.text)),
    );

    for (path, text) in texts {
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let held = line.trim_start();
            if held.is_empty() || held.starts_with('#') || held == "---" {
                continue;
            }
            let above = lines[..index]
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim_start());
            assert!(
                above.is_some_and(|above| above.starts_with('#')),
                "{path}: `{held}` has no comment above it"
            );
        }
    }
}

#[test]
fn every_shipped_block_can_be_switched_off_and_on_again_by_the_command_that_edits_it() {
    for shipped in COLLECTORS {
        for name in names(shipped.text) {
            let Switched::Changed(off) = switched(shipped.text, &name, false) else {
                panic!("{}: {name} cannot be switched off", shipped.path);
            };
            let blocks = blocks_in(Path::new(shipped.path), &off).expect("still parses");
            assert!(
                !blocks
                    .iter()
                    .find(|block| block.name == name)
                    .expect("the block")
                    .enabled,
                "{}: {off}",
                shipped.path
            );
            let Switched::Changed(on) = switched(&off, &name, true) else {
                panic!("{}: {name} cannot be switched back on", shipped.path);
            };
            assert!(
                blocks_in(Path::new(shipped.path), &on)
                    .expect("parses")
                    .iter()
                    .all(|block| block.enabled),
                "{}: {on}",
                shipped.path
            );
        }
    }
}

#[test]
fn a_host_where_nothing_can_run_still_writes_a_configuration_that_reads() {
    let directory = directory("nothing");
    let survey = vec![surveyed(
        "network",
        Health::Unavailable("no /proc on this host".into()),
    )];

    configure_with(&options(&directory), &survey).expect("writes");

    assert_eq!(loaded(&directory).collectors, Some(Vec::new()));
    assert!(!directory.join("collectors/network.yaml").exists());
    assert!(!directory.join(WATCH_LIST.path).exists());
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_file_that_is_already_there_is_not_touched_without_force_and_the_refusal_says_so() {
    let directory = directory("refused");
    std::fs::create_dir_all(directory.join("collectors")).expect("a directory");
    std::fs::write(
        directory.join("collectors/users.yaml"),
        "users:\n  schedule: 5\n",
    )
    .expect("writes");

    let refused = configure_with(&options(&directory), &survey()).expect_err("refused");

    assert!(refused.contains("--force"), "{refused}");
    assert_eq!(
        std::fs::read_to_string(directory.join("collectors/users.yaml")).expect("reads"),
        "users:\n  schedule: 5\n"
    );
    assert!(
        directory.join("collectors/network.yaml").exists(),
        "a file that is not there yet is written beside the one that is"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_file_that_already_says_what_would_be_written_is_left_as_it_is_and_refuses_nothing() {
    let directory = directory("unchanged");
    configure_with(&options(&directory), &survey()).expect("writes");
    let before = std::fs::metadata(directory.join("collectors/users.yaml"))
        .and_then(|metadata| metadata.modified())
        .expect("a file");

    configure_with(&options(&directory), &survey())
        .expect("a second run over its own files refuses nothing");

    assert_eq!(
        std::fs::metadata(directory.join("collectors/users.yaml"))
            .and_then(|metadata| metadata.modified())
            .expect("a file"),
        before
    );
    assert!(!directory.join("collectors/users.yaml.previous").exists());
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn with_force_what_was_there_is_kept_beside_the_new_file_as_previous() {
    let directory = directory("forced");
    std::fs::create_dir_all(directory.join("collectors")).expect("a directory");
    std::fs::write(
        directory.join("collectors/users.yaml"),
        "users:\n  schedule: 5\n",
    )
    .expect("writes");
    let forced = Options {
        force: true,
        ..options(&directory)
    };

    configure_with(&forced, &survey()).expect("writes");

    assert_eq!(
        std::fs::read_to_string(directory.join("collectors/users.yaml.previous")).expect("kept"),
        "users:\n  schedule: 5\n"
    );
    assert_eq!(loaded(&directory).every_seconds("users"), 300);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn with_force_the_file_of_a_collector_that_cannot_run_here_is_set_aside_and_read_no_more() {
    let directory = directory("aside");
    std::fs::create_dir_all(directory.join("collectors")).expect("a directory");
    std::fs::write(
        directory.join("collectors/launches.yaml"),
        "launches:\n  schedule: 15\n",
    )
    .expect("writes");
    let forced = Options {
        force: true,
        ..options(&directory)
    };

    configure_with(&forced, &survey()).expect("writes");

    assert!(!directory.join("collectors/launches.yaml").exists());
    assert!(directory.join("collectors/launches.yaml.previous").exists());
    assert!(
        !loaded(&directory)
            .collectors
            .unwrap_or_default()
            .contains(&"launches".to_string())
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_dry_run_writes_nothing_at_all() {
    let directory = directory("dry");
    let dry = Options {
        dry_run: true,
        ..options(&directory)
    };

    configure_with(&dry, &every_one_runs()).expect("prints");

    assert_eq!(
        std::fs::read_dir(&directory).expect("reads").count(),
        0,
        "--dry-run left something behind"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn no_line_it_prints_is_wider_than_eighty_columns_or_broken_in_the_middle_of_a_path() {
    let long = "auditd is not running — program launches are not visible (there is no /var/log/audit/audit.log, and the audit plugin has left nothing at /var/lib/vigil/audit-spool)";
    let mut survey = every_one_runs();
    survey.push(surveyed("launches", Health::Unavailable(long.into())));
    let mut said = progress::survey(&survey);
    for file in plan(Path::new("/etc/vigil/vigil.yaml"), &survey) {
        for action in [
            Action::Write,
            Action::Unchanged,
            Action::Replace,
            Action::Keep,
            Action::SetAside,
            Action::Leave,
        ] {
            said.extend(progress::said(action, &file, false));
            said.extend(progress::said(action, &file, true));
        }
    }
    let said = said.join("\n");

    assert!(said.contains("/var/log/audit/audit.log,"), "{said}");
    assert!(said.contains("/var/lib/vigil/audit-spool)"), "{said}");
    for line in said.lines() {
        assert!(line.chars().count() <= WIDTH, "{line}");
    }
}

#[test]
fn every_shipped_file_is_one_or_more_documents_each_holding_one_collector() {
    for shipped in COLLECTORS {
        for document in documents(shipped.text) {
            let name = document.name.expect("a block");
            assert_eq!(
                blocks_in(Path::new(shipped.path), &document.text)
                    .expect("parses")
                    .len(),
                1,
                "{}: {name}",
                shipped.path
            );
        }
    }
}
