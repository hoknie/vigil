use vigil_store::Store;

use crate::config::former_name;
use crate::loops::Watch;
use crate::modules::names;

pub fn forget(store: &dyn Store, switched_off: &[String]) {
    for name in switched_off {
        match store.forget_baseline(name) {
            Ok(true) => eprintln!("  baseline {name}: dropped, the collector is off"),
            Ok(false) => {}
            Err(error) => eprintln!("  baseline {name}: could not be dropped ({error})"),
        }
    }
    for current in names() {
        let Some(former) = former_name(current) else {
            continue;
        };
        match store.forget_baseline(former) {
            Ok(true) => eprintln!(
                "  baseline {former}: dropped, the collector is called {current} now and its \
                 first reading becomes its baseline"
            ),
            Ok(false) => {}
            Err(error) => eprintln!("  baseline {former}: could not be dropped ({error})"),
        }
    }
}

pub fn restore(store: &dyn Store, watches: &mut [Watch]) {
    for watch in watches {
        match store.baseline(watch.name()) {
            Ok(Some(baseline)) => {
                eprintln!(
                    "  baseline {}: continuing from the reading of {}",
                    watch.name(),
                    baseline.taken_at
                );
                watch.restore(baseline);
            }
            Ok(None) => eprintln!(
                "  baseline {}: none yet, the first reading becomes one",
                watch.name()
            ),
            Err(error) => eprintln!(
                "  baseline {}: unreadable ({error}), the first reading becomes one",
                watch.name()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use vigil_model::Snapshot;
    use vigil_store::FileStore;

    use super::*;

    fn temporary_directory(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "vigild-boot-baselines-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    fn reading(source: &str) -> Snapshot {
        let mut reading = Snapshot::new(source, "2026-09-17T12:00:00.000Z");
        reading
            .items
            .insert("tcp|0.0.0.0:22".to_string(), serde_json::json!({}));
        reading
    }

    #[test]
    fn a_baseline_left_under_the_former_name_ports_is_dropped_at_start_and_network_reads_afresh() {
        let directory = temporary_directory("renamed");
        let store = FileStore::open(&directory).expect("opens");
        store.set_baseline(&reading("ports")).expect("kept");

        forget(&store, &[]);

        assert_eq!(
            store.baseline("ports").expect("readable"),
            None,
            "a baseline no collector of this build reads would sit in the state directory for \
             ever, and the one reading it back would be a person at an incident"
        );
        assert_eq!(
            store.baseline("network").expect("readable"),
            None,
            "the socket collector is not handed the old baseline: its first reading under the \
             new name is a baseline of its own and raises nothing"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn dropping_the_former_name_leaves_the_baseline_of_network_where_it_is() {
        let directory = temporary_directory("kept");
        let store = FileStore::open(&directory).expect("opens");
        store.set_baseline(&reading("network")).expect("kept");

        forget(&store, &[]);

        assert_eq!(
            store.baseline("network").expect("readable"),
            Some(reading("network")),
            "a restart after the upgrade continues from the reading taken under the new name"
        );
        let _ = fs::remove_dir_all(&directory);
    }
}
