use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::{Value, json};

use super::hex::hex;
use super::hmac::hmac_sha256;

const FILE: &str = "installed_on";

const HARDWARE_APPLICATION_ID: &[u8] = b"vigil.host-findings.hardware-id.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    pub host_id: String,
    pub hardware_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    pub host_id: bool,
    pub hardware: bool,
}

impl Change {
    pub fn said(self) -> &'static str {
        match (self.host_id, self.hardware) {
            (true, true) => "the host id and the hardware identifier changed",
            (true, false) => "the host id changed",
            _ => "the hardware identifier changed under the same host id",
        }
    }
}

pub fn hardware_id(raw: &[u8]) -> String {
    hex(&hmac_sha256(raw, HARDWARE_APPLICATION_ID)[..16])
}

pub fn changed(recorded: &Machine, current: &Machine) -> Option<Change> {
    let change = Change {
        host_id: recorded.host_id != current.host_id,
        hardware: matches!(
            (&recorded.hardware_id, &current.hardware_id),
            (Some(before), Some(now)) if before != now
        ),
    };

    match change.host_id || change.hardware {
        true => Some(change),
        false => None,
    }
}

pub fn read(state_dir: &Path) -> Option<Machine> {
    let text = fs::read_to_string(state_dir.join(FILE)).ok()?;
    let value: Value = serde_json::from_str(text.trim()).ok()?;
    let host_id = value.get("host_id")?.as_str()?.to_string();
    let hardware_id = match value.get("hardware_id")? {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        _ => return None,
    };

    Some(Machine {
        host_id,
        hardware_id,
    })
}

pub fn write(state_dir: &Path, machine: &Machine) -> Result<(), String> {
    let path = state_dir.join(FILE);
    let temporary = state_dir.join(format!("{FILE}.writing"));
    let line = format!(
        "{}\n",
        json!({"host_id": machine.host_id, "hardware_id": machine.hardware_id})
    );

    let mut file = vigil_store::create_owner_only(&temporary, false)
        .map_err(|error| format!("{} cannot be written: {error}", temporary.display()))?;
    file.write_all(line.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("{} cannot be written: {error}", temporary.display()))?;
    drop(file);

    fs::rename(&temporary, &path)
        .map_err(|error| format!("{} cannot be written: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bench(name: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("vigil-installed-on-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a directory");
        directory
    }

    fn machine(host_id: &str, hardware_id: Option<&str>) -> Machine {
        Machine {
            host_id: host_id.to_string(),
            hardware_id: hardware_id.map(str::to_string),
        }
    }

    #[test]
    fn the_machine_an_installation_was_set_up_on_is_kept_owner_only() {
        let directory = bench("kept");
        let set_up_on = machine("a1", Some("b2"));

        write(&directory, &set_up_on).expect("writes");

        assert_eq!(read(&directory), Some(set_up_on));
        assert!(!directory.join("installed_on.writing").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(directory.join(FILE))
                .expect("stat")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600, "mode was {:o}", mode & 0o777);
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_raw_hardware_identifier_is_never_written() {
        let directory = bench("raw");
        let raw = "4c4c4544-0042-3510-8051-b4c04f4a3632";

        write(
            &directory,
            &machine("a1", Some(&hardware_id(raw.as_bytes()))),
        )
        .expect("writes");

        let written = fs::read_to_string(directory.join(FILE)).expect("readable");
        assert!(!written.contains(raw), "{written}");
        assert!(!written.contains("4c4c4544"), "{written}");
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_hardware_hash_cannot_be_matched_to_the_host_id() {
        let raw = b"4c4c4544-0042-3510-8051-b4c04f4a3632";

        assert_ne!(
            hardware_id(raw),
            super::super::host::derive_host_id(raw),
            "one value hashed for two uses gives two values nobody can line up"
        );
    }

    #[test]
    fn a_new_host_id_under_the_same_install_is_a_move() {
        let change = changed(&machine("a1", None), &machine("c3", None)).expect("a move");

        assert!(change.host_id && !change.hardware);
    }

    #[test]
    fn a_new_hardware_identifier_under_the_same_host_id_is_a_move() {
        let change =
            changed(&machine("a1", Some("b2")), &machine("a1", Some("d4"))).expect("a move");

        assert!(!change.host_id && change.hardware);
        assert!(change.said().contains("hardware identifier"));
    }

    #[test]
    fn a_hardware_identifier_that_appeared_or_vanished_is_not_a_move() {
        assert_eq!(
            changed(&machine("a1", None), &machine("a1", Some("b2"))),
            None
        );
        assert_eq!(
            changed(&machine("a1", Some("b2")), &machine("a1", None)),
            None
        );
    }

    #[test]
    fn the_same_machine_is_not_a_move() {
        assert_eq!(
            changed(&machine("a1", Some("b2")), &machine("a1", Some("b2"))),
            None
        );
    }
}
