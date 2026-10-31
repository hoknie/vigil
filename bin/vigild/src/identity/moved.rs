use std::path::Path;

use vigil_model::Uuid7;

use super::installed_on::{self, Change, Machine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    pub previous_install_id: Uuid7,
    pub install_id: Option<Uuid7>,
    pub previous_host_id: String,
    pub host_id: String,
    pub change: Change,
}

pub struct Remembered {
    pub install_id: Uuid7,
    pub moved: Option<Moved>,
    pub said: Vec<String>,
}

pub fn remember(
    state_dir: &Path,
    install_id: Uuid7,
    current: &Machine,
    mint: impl FnOnce(&Path) -> Result<Uuid7, String>,
) -> Remembered {
    let mut said = Vec::new();
    let recorded = installed_on::read(state_dir);
    let change = recorded
        .as_ref()
        .and_then(|recorded| installed_on::changed(recorded, current));

    let Some(change) = change else {
        if let Err(error) = installed_on::write(state_dir, current) {
            said.push(format!(
                "  identity: {error}; the next start compares with what was there before"
            ));
        }
        return Remembered {
            install_id,
            moved: None,
            said,
        };
    };

    let previous_host_id = recorded
        .map(|recorded| recorded.host_id)
        .unwrap_or_default();
    let minted = match mint(state_dir) {
        Ok(minted) => {
            if let Err(error) = installed_on::write(state_dir, current) {
                said.push(format!(
                    "  identity: {error}; the next start says this move again"
                ));
            }
            Some(minted)
        }
        Err(error) => {
            said.push(format!(
                "  identity: a new install id could not be written ({error}); this start keeps \
                 install {install_id}, and the next one tries again"
            ));
            None
        }
    };

    said.push(match &minted {
        Some(minted) => format!(
            "  identity: install {install_id} was set up on another machine ({}); the image was \
             probably cloned, and this one is now install {minted}",
            change.said()
        ),
        None => format!(
            "  identity: install {install_id} was set up on another machine ({}); the image was \
             probably cloned",
            change.said()
        ),
    });

    Remembered {
        install_id: minted.clone().unwrap_or_else(|| install_id.clone()),
        moved: Some(Moved {
            previous_install_id: install_id,
            install_id: minted,
            previous_host_id,
            host_id: current.host_id.clone(),
            change,
        }),
        said,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::identity::install::{install_id, replace};

    fn bench(name: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("vigil-moved-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    fn on(host_id: &str) -> Machine {
        Machine {
            host_id: host_id.to_string(),
            hardware_id: Some(format!("hardware-of-{host_id}")),
        }
    }

    fn start(directory: &Path, machine: &Machine) -> Remembered {
        let id = install_id(directory).expect("an install id");
        remember(directory, id, machine, replace)
    }

    #[test]
    fn an_upgrade_from_a_build_without_the_file_raises_nothing() {
        let directory = bench("upgrade");
        install_id(&directory).expect("a build before this one minted it");

        let started = start(&directory, &on("a1"));

        assert_eq!(started.moved, None);
        assert_eq!(installed_on::read(&directory), Some(on("a1")));
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_torn_file_is_rewritten_and_raises_nothing() {
        let directory = bench("torn");
        install_id(&directory).expect("minted");
        fs::write(directory.join("installed_on"), b"{\"host_id\":\"a1").expect("tears");

        let started = start(&directory, &on("c3"));

        assert_eq!(started.moved, None);
        assert_eq!(installed_on::read(&directory), Some(on("c3")));
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_moved_installation_gets_a_new_install_id() {
        let directory = bench("moved");
        let before = start(&directory, &on("a1")).install_id;

        let moved = start(&directory, &on("c3"));

        let said = moved.moved.clone().expect("a move");
        assert_eq!(said.previous_install_id, before);
        assert_eq!(said.install_id.as_deref(), Some(moved.install_id.as_str()));
        assert_ne!(
            moved.install_id, before,
            "two hosts from one image are told apart from their first document on"
        );
        assert_eq!(
            install_id(&directory).expect("read back"),
            moved.install_id,
            "and the new one is the one a restart reads"
        );
        assert!(
            moved.said.iter().any(|line| line.contains(&before)
                && line.contains(&moved.install_id)
                && line.contains("probably cloned")),
            "{:?}",
            moved.said
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_second_start_on_the_moved_machine_is_quiet_and_keeps_its_new_install_id() {
        let directory = bench("second");
        start(&directory, &on("a1"));
        let moved = start(&directory, &on("c3"));

        let again = start(&directory, &on("c3"));

        assert_eq!(again.moved, None);
        assert_eq!(again.install_id, moved.install_id);
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn an_install_id_that_cannot_be_written_keeps_the_old_one_and_tries_again() {
        let directory = bench("unwritable");
        let before = start(&directory, &on("a1")).install_id;

        let id = install_id(&directory).expect("read");
        let failed = remember(&directory, id, &on("c3"), |_| Err("disk full".to_string()));

        assert_eq!(failed.install_id, before);
        assert!(failed.moved.is_some(), "the move is still said");
        assert_eq!(
            installed_on::read(&directory),
            Some(on("a1")),
            "the machine is not recorded, so the next start says the move again"
        );
        let retried = start(&directory, &on("c3"));
        assert!(retried.moved.is_some());
        assert_ne!(retried.install_id, before);
        let _ = fs::remove_dir_all(&directory);
    }
}
