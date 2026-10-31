use std::path::Path;

use vigil_model::Host;

use super::fqdn;
use super::hex::hex;
use super::hmac::hmac_sha256;
use super::install::{install_id, replace};
use super::installed_on::{Machine, hardware_id};
#[cfg(not(target_os = "macos"))]
use super::linux as system;
#[cfg(target_os = "macos")]
use super::macos as system;
use super::moved::{Moved, remember};

const APPLICATION_ID: &[u8] = b"vigil.host-findings.host-id.v1";

#[cfg(not(target_os = "macos"))]
pub const HOST_ID_SOURCES: &[&str] = &["/etc/machine-id", "/var/lib/dbus/machine-id"];

pub struct Described {
    pub host: Host,
    pub moved: Option<Moved>,
    pub said: Vec<String>,
}

pub fn describe(state_dir: &Path) -> Result<Described, String> {
    let (raw_host_id, source) = system::host_id();
    let host_id = derive_host_id(&raw_host_id);
    let machine = Machine {
        host_id: host_id.clone(),
        hardware_id: system::hardware_id().map(|raw| hardware_id(&raw)),
    };
    let remembered = remember(state_dir, install_id(state_dir)?, &machine, replace);

    let mut tags = std::collections::BTreeMap::new();
    tags.insert("host_id_source".to_string(), source);

    let hostname = system::hostname();
    let host = Host {
        host_id,
        install_id: remembered.install_id,
        boot_id: system::boot_id(),
        fqdn: fqdn::read(&hostname),
        hostname,
        os: system::os(),
        addresses: Vec::new(),
        tags,
        peer: None,
    };

    Ok(Described {
        host,
        moved: remembered.moved,
        said: remembered.said,
    })
}

pub(super) fn derive_host_id(raw: &[u8]) -> String {
    hex(&hmac_sha256(raw, APPLICATION_ID)[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_published_identifier_is_not_the_system_one() {
        let raw = b"3f2a1c9d8e7b4a5c6d0e1f2a3b4c5d6e";

        let derived = derive_host_id(raw);

        assert_eq!(derived.len(), 32);
        assert!(
            !derived.contains("3f2a1c9d"),
            "the raw /etc/machine-id must not be recoverable from what we publish"
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn the_value_is_called_host_id_and_the_file_it_is_read_from_is_still_called_machine_id() {
        assert_eq!(
            HOST_ID_SOURCES,
            ["/etc/machine-id", "/var/lib/dbus/machine-id"],
            "host is what this product calls the node it watches; machine-id is the kernel's name for the file. Renaming the path reads a file that is not there, and every host silently falls back to its hostname"
        );
    }

    #[test]
    fn the_same_host_derives_the_same_identifier_every_time() {
        assert_eq!(derive_host_id(b"abc"), derive_host_id(b"abc"));
        assert_ne!(derive_host_id(b"abc"), derive_host_id(b"abd"));
    }

    #[test]
    fn the_host_this_build_runs_on_is_described_with_a_hostname_and_a_system() {
        let directory = std::env::temp_dir().join(format!("vigil-host-{}", std::process::id()));

        let host = describe(&directory).expect("described").host;

        assert_ne!(host.hostname, "unknown");
        assert_eq!(host.os.family, std::env::consts::OS);
        assert!(host.tags.contains_key("host_id_source"));
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn peer_is_never_filled_by_this_agent() {
        let directory =
            std::env::temp_dir().join(format!("vigil-host-peer-{}", std::process::id()));

        let host = describe(&directory).expect("described").host;

        assert!(
            host.peer.is_none(),
            "the node beside this host is known to another product, and this agent reads none of its files"
        );
        let _ = std::fs::remove_dir_all(&directory);
    }
}
