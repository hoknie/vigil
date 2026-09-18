use std::path::PathBuf;

use serde_json::json;
use vigil_module::{Module, Settings};

use crate::modules::Files;
use crate::types::{Devices, Layout, Listing, MAX_FILES, MOST_FILES, Watching};

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-18T12:00:00.000Z".to_string()
}

fn block(said: serde_json::Value) -> Settings {
    Settings::of(at_noon, "files", said)
}

#[test]
fn a_block_that_names_its_watch_list_watches_what_that_list_names() {
    let settings = block(json!({
        "watched_path": "/etc/vigil/watch_fs.yaml",
        "max_file_size": "30mb",
        "devices": { "include": [], "exclude": ["nfs"] },
    }));
    let watching: Watching = settings.read().expect("the sample parses");

    assert!(Files.check(&settings).is_ok());
    assert_eq!(
        watching.layout(),
        Layout::Listed(Listing {
            watched_path: PathBuf::from("/etc/vigil/watch_fs.yaml"),
            max_file_size: 30 * 1024 * 1024,
            devices: Devices {
                include: Vec::new(),
                exclude: vec!["nfs".to_string()],
            },
            max_files: MAX_FILES,
        })
    );
}

#[test]
fn a_walk_is_bounded_by_ten_thousand_paths_a_reading_unless_the_block_says_fewer() {
    assert_eq!(MAX_FILES, 10_000);
    assert_eq!(
        MOST_FILES, MAX_FILES,
        "ten thousand walked paths measured at about 60 MB resident with the reading before \
         them held for the comparison, and the whole agent is allowed 64 MB: a block may ask \
         for fewer and never for more"
    );

    let Layout::Listed(listing) = block(json!({
        "watched_path": "/etc/vigil/watch_fs.yaml",
        "max_files": 500,
    }))
    .read::<Watching>()
    .expect("parses")
    .layout() else {
        panic!("a block naming a watch list reads that list");
    };

    assert_eq!(listing.max_files, 500);
}

#[test]
fn a_block_that_mixes_the_old_keys_with_the_new_is_refused_in_a_sentence_naming_both() {
    let refusal = Files
        .check(&block(json!({
            "paths": ["/etc/hosts"],
            "watched_path": "/etc/vigil/watch_fs.yaml",
        })))
        .expect_err("one block, two lists: the agent would watch one of them");

    assert!(refusal.contains("paths"), "{refusal}");
    assert!(refusal.contains("watched_path"), "{refusal}");
    assert!(refusal.contains("Keep one"), "{refusal}");
}

#[test]
fn a_host_upgraded_with_the_old_keys_in_its_configuration_goes_on_watching_what_they_name() {
    let settings = block(json!({
        "paths": ["/etc/hosts", { "path": "/etc/ssl/certs/ca.crt", "ceiling_bytes": 4096 }],
        "ceiling_bytes": 2048,
    }));

    assert!(Files.check(&settings).is_ok());
    assert_eq!(
        settings.read::<Watching>().expect("parses").hashed(),
        vec![
            ("/etc/hosts".to_string(), 2048),
            ("/etc/ssl/certs/ca.crt".to_string(), 4096),
        ]
    );
}

#[test]
fn a_block_the_agent_could_not_act_on_is_refused_at_the_door_in_the_key_it_is_written_in() {
    for (said, complained_about) in [
        (json!({"watched_path": "watch_fs.yaml"}), "absolute"),
        (
            json!({"watched_path": "/w", "max_file_size": 0}),
            "max_file_size: 0",
        ),
        (
            json!({"watched_path": "/w", "max_file_size": "1gb"}),
            "over the 64mb",
        ),
        (
            json!({"watched_path": "/w", "max_file_size": "lots"}),
            "30mb",
        ),
        (
            json!({"watched_path": "/w", "max_files": 0}),
            "max_files: 0",
        ),
        (
            json!({"watched_path": "/w", "max_files": 10_001}),
            "max_files",
        ),
        (
            json!({"watched_path": "/w", "devices": {"include": [""]}}),
            "devices.include",
        ),
        (
            json!({"watched_path": "/w", "devices": {"only": ["nfs"]}}),
            "only",
        ),
        (
            json!({"watched_path": "/w", "source_path": "/w"}),
            "source_path",
        ),
    ] {
        let refusal = Files
            .check(&block(said.clone()))
            .expect_err("a block this collector cannot act on is one nobody meant to write");

        assert!(refusal.contains(complained_about), "{said}: {refusal}");
    }
}

#[test]
fn a_size_in_the_block_is_the_size_every_entry_of_the_list_without_one_is_hashed_to() {
    let Layout::Named(named) = block(json!({ "max_file_size": "512kb" }))
        .read::<Watching>()
        .expect("parses")
        .layout()
    else {
        panic!("a block that names no list watches the shipped one");
    };

    assert_eq!(named.ceiling_bytes, 512 * 1024);
}
