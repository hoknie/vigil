use serde_json::{Value, json};
use vigil_model::Snapshot;

pub const SOURCE: &str = "files";

pub const FILE: &str = "file";

pub const DIRECTORY: &str = "directory";

pub const WALK: &str = "walk";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchedFile {
    pub path: String,
    pub present: bool,
    pub readable: bool,
    pub digest: Option<String>,
    pub size: u64,
    pub mode: Option<String>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub over_the_ceiling: bool,
    pub ceiling_bytes: u64,
    pub found: Found,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    pub kind: Option<&'static str>,
    pub target: Option<String>,
    pub walked: Option<Walked>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walked {
    pub by: String,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkRow {
    pub entry: String,
    pub kind: &'static str,
    pub matched: usize,
    pub complete: bool,
    pub not_entered: Vec<String>,
    pub max_file_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchedDirectory {
    pub path: String,
    pub present: bool,
    pub mode: Option<String>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

pub struct FilesReading<'a> {
    pub files: &'a [WatchedFile],
    pub directories: &'a [WatchedDirectory],
    pub walks: &'a [WalkRow],
}

pub fn files_snapshot(taken_at: &str, reading: &FilesReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    for file in reading.files {
        snapshot
            .items
            .insert(format!("{FILE}|{}", file.path), row_of(file));
    }

    for directory in reading.directories {
        snapshot.items.insert(
            format!("{DIRECTORY}|{}", directory.path),
            json!({
                "path": directory.path,
                "present": directory.present,
                "mode": directory.mode,
                "uid": directory.uid,
                "gid": directory.gid,
            }),
        );
    }

    for walk in reading.walks {
        snapshot.items.insert(
            format!("{WALK}|{}", walk.entry),
            json!({
                "entry": walk.entry,
                "kind": walk.kind,
                "matched": walk.matched,
                "complete": walk.complete,
                "not_entered": walk.not_entered,
                "max_file_size": walk.max_file_size,
            }),
        );
    }

    snapshot
}

fn row_of(file: &WatchedFile) -> Value {
    let mut row = json!({
        "path": file.path,
        "present": file.present,
        "readable": file.readable,
        "sha256": file.digest,
        "size": file.size,
        "mode": file.mode,
        "uid": file.uid,
        "gid": file.gid,
        "over_the_ceiling": file.over_the_ceiling,
        "ceiling_bytes": file.ceiling_bytes,
    });
    if let Some(kind) = file.found.kind {
        row["type"] = json!(kind);
    }
    if let Some(target) = &file.found.target {
        row["target"] = json!(target);
    }
    if let Some(walked) = &file.found.walked {
        row["found_by"] = json!(walked.by);
        row["complete"] = json!(walked.complete);
    }
    row
}

#[cfg(test)]
mod tests {
    use vigil_model::class_of;

    use super::*;

    const AT: &str = "2026-09-11T12:00:00.000Z";

    fn watched(path: &str) -> WatchedFile {
        WatchedFile {
            path: path.to_string(),
            present: true,
            readable: true,
            digest: Some("a9".repeat(32)),
            size: 3_281,
            mode: Some("0644".into()),
            uid: Some(0),
            gid: Some(0),
            over_the_ceiling: false,
            ceiling_bytes: 1024 * 1024,
            found: Found::default(),
        }
    }

    fn gone(path: &str) -> WatchedFile {
        WatchedFile {
            path: path.to_string(),
            present: false,
            readable: false,
            digest: None,
            size: 0,
            mode: None,
            uid: None,
            gid: None,
            over_the_ceiling: false,
            ceiling_bytes: 1024 * 1024,
            found: Found::default(),
        }
    }

    fn directory(path: &str, mode: &str) -> WatchedDirectory {
        WatchedDirectory {
            path: path.to_string(),
            present: true,
            mode: Some(mode.to_string()),
            uid: Some(0),
            gid: Some(0),
        }
    }

    fn reading(files: &[WatchedFile], directories: &[WatchedDirectory]) -> Snapshot {
        files_snapshot(
            AT,
            &FilesReading {
                files,
                directories,
                walks: &[],
            },
        )
    }

    #[test]
    fn a_watched_file_and_a_directory_of_the_path_are_two_classes_of_row() {
        let taken = reading(
            &[watched("/etc/ssh/sshd_config")],
            &[directory("/usr/local/bin", "0755")],
        );

        let mut classes: Vec<&str> = taken.items.keys().map(|key| class_of(key)).collect();
        classes.sort_unstable();

        assert_eq!(classes, vec!["directory", "file"]);
        assert_eq!(taken.source, SOURCE);
        assert_eq!(taken.items["file|/etc/ssh/sshd_config"]["mode"], "0644");
    }

    #[test]
    fn a_path_that_is_not_there_is_a_row_saying_so_and_never_a_row_that_is_missing() {
        let taken = reading(&[gone("/etc/ssh/sshd_config")], &[]);
        let row = &taken.items["file|/etc/ssh/sshd_config"];

        assert_eq!(row["present"], false);
        assert_eq!(row["sha256"], serde_json::Value::Null);
        assert_eq!(
            taken.items.len(),
            1,
            "a watched path that vanished from the reading would read as a path nobody asked \
             about, and the reader would never learn it is gone"
        );
    }

    #[test]
    fn the_moment_a_file_was_last_written_is_nowhere_in_the_reading() {
        let taken = reading(&[watched("/etc/hosts")], &[]);
        let row = &taken.items["file|/etc/hosts"];

        assert!(
            row.get("mtime").is_none() && row.get("modified_at").is_none(),
            "a file rewritten with the same content by a configuration manager moves its \
             timestamp and nothing else, and a reading carrying that is a finding on every run \
             of the manager"
        );
    }

    #[test]
    fn a_file_too_big_to_hash_is_a_row_that_says_so_rather_than_a_row_without_a_digest() {
        let mut huge = watched("/etc/hosts");
        huge.digest = None;
        huge.over_the_ceiling = true;
        huge.size = 64 * 1024 * 1024;

        let taken = reading(&[huge], &[]);
        let row = &taken.items["file|/etc/hosts"];

        assert_eq!(row["over_the_ceiling"], true);
        assert_eq!(row["sha256"], serde_json::Value::Null);
        assert_eq!(row["present"], true);
    }

    #[test]
    fn a_file_named_in_the_list_is_written_exactly_as_it_was_before_walks_were_read() {
        let taken = reading(&[watched("/etc/hosts")], &[]);
        let row = taken.items["file|/etc/hosts"]
            .as_object()
            .expect("a row is an object");

        let mut fields: Vec<&str> = row.keys().map(String::as_str).collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            vec![
                "ceiling_bytes",
                "gid",
                "mode",
                "over_the_ceiling",
                "path",
                "present",
                "readable",
                "sha256",
                "size",
                "uid",
            ],
            "a host whose list did not change keeps the rows it held, or every one of them is a \
             change on the first reading after an upgrade"
        );
    }

    #[test]
    fn a_file_found_by_a_walk_says_which_entry_found_it_and_whether_that_walk_was_whole() {
        let mut found = watched("/etc/pam.d/common-auth");
        found.found = Found {
            kind: Some("file"),
            target: None,
            walked: Some(Walked {
                by: "/etc/pam.d".into(),
                complete: true,
            }),
        };
        let walk = WalkRow {
            entry: "/etc/pam.d".into(),
            kind: "tree",
            matched: 1,
            complete: true,
            not_entered: Vec::new(),
            max_file_size: 1024,
        };

        let taken = files_snapshot(
            AT,
            &FilesReading {
                files: &[found],
                directories: &[],
                walks: &[walk],
            },
        );

        let row = &taken.items["file|/etc/pam.d/common-auth"];
        assert_eq!(row["found_by"], "/etc/pam.d");
        assert_eq!(row["complete"], true);
        assert_eq!(row["type"], "file");
        assert_eq!(taken.items["walk|/etc/pam.d"]["matched"], 1);
        assert_eq!(class_of("walk|/etc/pam.d"), WALK);
    }
}
