use std::path::PathBuf;

use crate::stores::files::batch::{BatchFile, BatchRecord};
use crate::stores::files::{Limits, Outgoing};

use super::harness::temporary_directory;

fn record() -> BatchRecord {
    BatchRecord {
        batch_id: "0192f0a0-0000-7000-8000-000000000001".into(),
        findings: 3,
        first_event_id: "first".into(),
        last_event_id: "last".into(),
    }
}

fn buffer_in(name: &str) -> (PathBuf, PathBuf) {
    let directory = temporary_directory(name);
    std::fs::create_dir_all(&directory).expect("makes the directory");
    let buffer = directory.join("ndjson.ndjson");
    (directory, buffer)
}

#[test]
fn a_held_batch_is_written_owner_only_and_atomically() {
    let (directory, buffer) = buffer_in("written");
    let file = BatchFile::beside(&buffer);

    file.write(&record()).expect("writes");

    assert_eq!(file.read(), Some(record()));
    let written = directory.join("ndjson.batch");
    assert!(
        written.exists(),
        "the batch sits beside the buffer it belongs to"
    );
    assert!(
        !directory.join("ndjson.batch.writing").exists(),
        "what a reader finds is the old file or the new one, never one half written"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&written)
            .expect("stat")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "mode was {:o}", mode & 0o777);
    }
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn an_unreadable_batch_file_is_forgotten_and_does_not_stop_the_buffer() {
    let (directory, buffer) = buffer_in("unreadable");
    let file = BatchFile::beside(&buffer);
    std::fs::write(
        directory.join("ndjson.batch"),
        b"{\"batch_id\":\"0192f0a0-00",
    )
    .expect("tears");

    assert_eq!(
        file.read(),
        None,
        "a torn file names no batch rather than a wrong one"
    );
    let opened = Outgoing::open("ndjson", buffer, Limits::outgoing())
        .expect("a torn batch file does not stop the buffer from opening");
    assert_eq!(opened.held_batch(), None);
    assert!(
        !directory.join("ndjson.batch").exists(),
        "and it is removed rather than read again at every start"
    );
    let _ = std::fs::remove_dir_all(&directory);
}
