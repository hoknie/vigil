use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use vigil_collect::{Collector, Health, outside_the_sample, this_account};
use vigil_model::{Golden, Shape, Snapshot};
use vigil_rules::{RuleContext, diff};

use super::FilesCollector;
use crate::rules::file_rules;
use crate::types::{Devices, Listing};

struct Bench {
    directory: PathBuf,
}

impl Bench {
    fn new(named: &str) -> Bench {
        let directory = std::env::temp_dir().join(format!(
            "vigil-files-macos-{named}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a bench to read from");
        Bench { directory }
    }

    fn at(&self, name: &str) -> String {
        self.directory.join(name).display().to_string()
    }

    fn write(&self, name: &str, text: &str) -> String {
        let at = self.directory.join(name);
        if let Some(parent) = at.parent() {
            fs::create_dir_all(parent).expect("a directory to write in");
        }
        fs::write(&at, text).expect("writes");
        fs::set_permissions(&at, fs::Permissions::from_mode(0o644)).expect("sets the mode");
        at.display().to_string()
    }

    fn named(&self, watched: &[String]) -> FilesCollector {
        let hashed: Vec<(String, u64)> = watched.iter().map(|path| (path.clone(), 4096)).collect();
        FilesCollector::with_directories(
            || "2026-09-19T12:00:00.000Z".to_string(),
            &hashed,
            &["/usr/bin", "/usr/local/bin"],
        )
    }

    fn listed(&self, list: &str) -> FilesCollector {
        FilesCollector::listed_with_directories(
            || "2026-09-19T12:00:00.000Z".to_string(),
            Listing {
                watched_path: self.directory.join(list),
                max_file_size: 1024 * 1024,
                devices: Devices::default(),
                max_files: 100,
            },
            &[],
        )
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn judged(before: &Snapshot, after: &Snapshot) -> Vec<vigil_model::Finding> {
    let mut minted = 0;
    let mut mint = || {
        minted += 1;
        format!("event-{minted}")
    };
    let mut ctx = RuleContext {
        now: "2026-09-19T12:05:00.000Z".into(),
        mint_event_id: &mut mint,
    };
    file_rules().judge(&diff(before, after), &mut ctx)
}

#[test]
fn a_watched_file_on_a_mac_is_read_with_its_content_its_mode_and_its_owner() {
    let bench = Bench::new("one-file");
    let path = bench.write("sshd_config", "PermitRootLogin no\n");

    let reading = bench
        .named(std::slice::from_ref(&path))
        .collect()
        .expect("reads");

    let row = &reading.items[&format!("file|{path}")];
    assert_eq!(row["present"], true);
    assert_eq!(row["readable"], true);
    assert_eq!(row["mode"], "0644");
    assert_eq!(row["uid"], this_account());
    assert_eq!(row["sha256"].as_str().map(str::len), Some(64));
    assert_eq!(reading.items["directory|/usr/bin"]["present"], true);
}

#[test]
fn a_file_dropped_into_a_watched_directory_of_a_mac_is_a_finding() {
    let bench = Bench::new("walk-finding");
    bench.write("etc/pam.d/sudo", "auth sufficient pam_smartcard.so\n");
    let root = bench.at("etc/pam.d");
    bench.write("watch_fs.yaml", &format!("files:\n  - {root}\n"));
    let collector = bench.listed("watch_fs.yaml");

    let before = collector.collect().expect("reads");
    bench.write("etc/pam.d/backdoor", "auth sufficient pam_permit.so\n");
    let after = collector.collect().expect("reads");

    let findings = judged(&before, &after);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].finding_key, format!("file|{root}/backdoor"));
    assert_eq!(
        after.items[&format!("walk|{root}")]["complete"],
        true,
        "{}",
        after.items[&format!("walk|{root}")]
    );
}

#[test]
fn the_devices_of_the_kernel_of_a_mac_are_never_walked_even_when_the_list_names_them() {
    let bench = Bench::new("walk-dev");
    bench.write("watch_fs.yaml", "files:\n  - /dev\n  - /dev/tty*\n");

    let reading = bench.listed("watch_fs.yaml").collect().expect("reads");

    let walked = &reading.items["walk|/dev"];
    assert_eq!(walked["matched"], 0, "{walked}");
    assert!(
        walked["not_entered"][0]
            .as_str()
            .is_some_and(|said| said.contains("(devfs)")),
        "{walked}"
    );
    assert_eq!(reading.items["walk|/dev/tty*"]["matched"], 0);
}

#[test]
fn a_walk_through_a_path_of_the_system_volume_into_the_data_volume_is_complete() {
    let bench = Bench::new("walk-firmlink");
    bench.write("watch_fs.yaml", "files:\n  - /private/etc/ssh\n");

    let reading = bench.listed("watch_fs.yaml").collect().expect("reads");

    let walked = &reading.items["walk|/private/etc/ssh"];
    assert_eq!(walked["complete"], true, "{walked}");
    assert!(
        reading
            .items
            .contains_key("file|/private/etc/ssh/sshd_config"),
        "{:?}",
        reading.items.keys().collect::<Vec<_>>()
    );
}

#[test]
fn a_directory_named_through_the_link_at_etc_is_walked_under_the_name_the_list_gave() {
    let bench = Bench::new("walk-etc");
    bench.write("watch_fs.yaml", "files:\n  - /etc/ssh\n");

    let reading = bench.listed("watch_fs.yaml").collect().expect("reads");

    assert_eq!(reading.items["walk|/etc/ssh"]["complete"], true);
    assert_eq!(reading.items["file|/etc/ssh"]["type"], "directory");
    assert!(reading.items.contains_key("file|/etc/ssh/sshd_config"));
}

#[test]
fn a_reading_of_macos_has_the_shape_of_the_reading_the_rules_and_the_console_were_built_on() {
    let bench = Bench::new("shape");
    bench.write("etc/pam.d/sudo", "auth sufficient pam_smartcard.so\n");
    bench.write("etc/ssh/sshd_config", "PermitRootLogin no\n");
    let root = bench.at("etc/pam.d");
    let named = bench.at("etc/ssh/sshd_config");
    let mask = bench.at("etc/ssh/*_config");
    bench.write(
        "watch_fs.yaml",
        &format!("files:\n  - {root}\n  - {named}\n  - {mask}\n  - /etc/shadow\n"),
    );
    let sample: Shape = serde_json::from_str(
        &Golden::snapshot("files")
            .held()
            .expect("the published shape of files"),
    )
    .expect("a shape");

    let reading = bench.listed("watch_fs.yaml").collect().expect("reads");

    let drift = outside_the_sample(&Shape::of(&reading), &sample, &[]);
    assert!(drift.is_empty(), "{drift:#?}");
}

#[test]
fn a_list_that_names_only_readable_places_leaves_the_collector_well() {
    let bench = Bench::new("health");
    bench.write("etc/pam.d/sudo", "auth sufficient pam_smartcard.so\n");
    let root = bench.at("etc/pam.d");
    bench.write("watch_fs.yaml", &format!("files:\n  - {root}\n"));
    let collector = bench.listed("watch_fs.yaml");

    let _ = collector.collect().expect("reads");

    assert_eq!(collector.available(), Health::Ok);
}
