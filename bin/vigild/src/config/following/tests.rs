use std::fs;
use std::path::PathBuf;

use serde_json::json;

use super::{Followed, Stamp};
use crate::config::load;

const WATCHING_HOSTS: &str = "files:\n  paths:\n    - \"/etc/hosts\"\n";

const WATCHING_SUDOERS: &str = "files:\n  paths:\n    - \"/etc/hosts\"\n    - \"/etc/sudoers\"\n";

struct Bench {
    directory: PathBuf,
    path: String,
}

impl Bench {
    fn new(named: &str, text: &str) -> Bench {
        let directory = std::env::temp_dir().join(format!(
            "vigild-following-{named}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&directory).expect("a directory to write in");
        let path = directory.join("vigil.yaml").display().to_string();
        let bench = Bench { directory, path };
        bench.write(text);
        bench
    }

    fn write(&self, text: &str) {
        vigil_config::write(std::path::Path::new(&self.path), text, true)
            .expect("writes the way the console writes");
    }

    fn followed(&self, watched: &[&str]) -> Followed {
        let stamp = Stamp::of(&self.path);
        let config = load(&self.path).expect("the file the daemon started from loads");

        Followed::of(
            &self.path,
            stamp,
            &config,
            crate::modules::modules(),
            watched,
        )
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn a_path_added_to_the_file_is_handed_to_the_files_module_on_the_next_look() {
    let bench = Bench::new("good", WATCHING_HOSTS);
    let mut followed = bench.followed(&["files"]);

    bench.write(WATCHING_SUDOERS);
    let looked = followed.look();

    assert_eq!(looked.refollowed.len(), 1, "{:?}", looked.said);
    let (name, settings) = &looked.refollowed[0];
    assert_eq!(*name, "files");
    assert_eq!(
        settings.said(),
        &json!({"paths": ["/etc/hosts", "/etc/sudoers"]}),
        "a path the console wrote is a path the running daemon watches, without a restart"
    );
    assert!(
        looked.said.is_empty(),
        "a good edit to the watched paths is not a complaint: {:?}",
        looked.said
    );
}

#[test]
fn a_file_nobody_touched_since_the_last_look_hands_nothing_over_and_says_nothing() {
    let bench = Bench::new("untouched", WATCHING_HOSTS);
    let mut followed = bench.followed(&["files"]);

    for _ in 0..3 {
        let looked = followed.look();

        assert!(
            looked.refollowed.is_empty() && looked.said.is_empty(),
            "a round of a daemon whose file did not change costs a stat and nothing more: {:?}",
            looked.said
        );
    }
}

#[test]
fn a_bad_edit_keeps_the_last_good_list_and_is_said_once_rather_than_every_round() {
    let bench = Bench::new("bad", WATCHING_HOSTS);
    let mut followed = bench.followed(&["files"]);

    for broken in [
        "files:\n  paths: [\n",
        "files:\n  paths:\n    - \"etc/sudoers\"\n",
    ] {
        bench.write(broken);

        let looked = followed.look();
        assert!(
            looked.refollowed.is_empty(),
            "{broken:?}: a file the daemon would refuse to start from hands no list to a \
             running module, so the list it watched stays watched"
        );
        assert_eq!(looked.said.len(), 1, "{broken:?}: {:?}", looked.said);
        assert!(
            looked.said[0].contains(&bench.path) && looked.said[0].contains("last loaded"),
            "the line names the file and says what is still watched: {}",
            looked.said[0]
        );

        let again = followed.look();
        assert!(
            again.said.is_empty() && again.refollowed.is_empty(),
            "{broken:?}: the same bad file on the next round is already said: {:?}",
            again.said
        );
    }
}

#[test]
fn a_file_that_loads_again_after_a_bad_edit_is_taken_up_and_said_to_load_again() {
    let bench = Bench::new("recovered", WATCHING_HOSTS);
    let mut followed = bench.followed(&["files"]);
    bench.write("files:\n  celing_bytes: 2048\n");
    followed.look();

    bench.write(WATCHING_SUDOERS);
    let looked = followed.look();

    assert!(
        looked.said.iter().any(|line| line.contains("loads again")),
        "an operator who saw the complaint is told it is over: {:?}",
        looked.said
    );
    assert_eq!(
        looked
            .refollowed
            .iter()
            .map(|(name, settings)| (*name, settings.said().clone()))
            .collect::<Vec<_>>(),
        vec![("files", json!({"paths": ["/etc/hosts", "/etc/sudoers"]}))]
    );
}

#[test]
fn a_file_that_goes_away_keeps_what_is_watched_and_says_so_once() {
    let bench = Bench::new("gone", WATCHING_HOSTS);
    let mut followed = bench.followed(&["files"]);
    fs::remove_file(&bench.path).expect("removes");

    let looked = followed.look();
    let again = followed.look();

    assert!(looked.refollowed.is_empty());
    assert_eq!(looked.said.len(), 1, "{:?}", looked.said);
    assert!(
        looked.said[0].contains("cannot be read"),
        "{}",
        looked.said[0]
    );
    assert!(again.said.is_empty(), "{:?}", again.said);
}

#[test]
fn a_console_switch_turned_on_in_the_file_while_the_daemon_runs_stays_as_the_daemon_started() {
    let bench = Bench::new("switches", WATCHING_HOSTS);
    let mut followed = bench.followed(&["files"]);

    bench.write(&format!(
        "{WATCHING_HOSTS}killing:\n  from_the_console: true\naccounts:\n  from_the_console: \
         true\nunits:\n  from_the_console: true\n"
    ));
    let looked = followed.look();

    assert!(
        looked.refollowed.is_empty(),
        "killing, accounts and units are read once, at start-up, on purpose: the console says \
         in words what it may do to this host, and a word it said at start that an edit could \
         take back or hand out while the daemon runs is a word nobody can rely on"
    );
    assert_eq!(looked.said.len(), 1, "{:?}", looked.said);
    for key in ["killing", "accounts", "units", "try-restart"] {
        assert!(
            looked.said[0].contains(key),
            "the operator is told the switch waits for a restart, by name: {}",
            looked.said[0]
        );
    }
}

#[test]
fn a_block_of_a_module_that_does_not_follow_the_file_waits_for_a_restart_and_is_said_to() {
    let bench = Bench::new("resources", WATCHING_HOSTS);
    let mut followed = bench.followed(&["files", "resources"]);

    bench.write(&format!(
        "{WATCHING_HOSTS}resources:\n  disk_free_percent: 20\n"
    ));
    let looked = followed.look();

    assert!(looked.refollowed.is_empty(), "{:?}", looked.said);
    assert!(
        looked.said.iter().any(|line| line.contains("resources")),
        "{:?}",
        looked.said
    );
}

#[test]
fn only_the_watched_paths_are_taken_from_the_file_while_the_daemon_runs() {
    let bench = Bench::new("who", WATCHING_HOSTS);
    let everything: Vec<&str> = crate::modules::names();

    assert_eq!(
        bench.followed(&everything).names(),
        vec!["files"],
        "a module added to this list is a block of vigil.yaml an edit reaches without a \
         restart, and that is a decision to make on purpose, with a test of its own"
    );
}

#[test]
fn a_host_where_the_files_collector_is_off_looks_at_nothing_on_any_round() {
    let bench = Bench::new("off", WATCHING_HOSTS);
    let mut followed = bench.followed(&["ports", "users"]);
    bench.write(WATCHING_SUDOERS);

    let looked = followed.look();

    assert!(followed.names().is_empty());
    assert!(
        looked.refollowed.is_empty() && looked.said.is_empty(),
        "a collector that does not run is handed nothing, and a daemon that follows nothing \
         does not stat the file either: {:?}",
        looked.said
    );
}
