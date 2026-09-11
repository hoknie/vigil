use std::fs;
use std::path::Path;

use vigil_model::{Host, Os};

use super::hex::hex;
use super::hmac::hmac_sha256;
use super::install::install_id;

const APPLICATION_ID: &[u8] = b"vigil.host-findings.host-id.v1";

const HOST_ID_SOURCES: &[&str] = &["/etc/machine-id", "/var/lib/dbus/machine-id"];

pub fn describe(state_dir: &Path) -> Result<Host, String> {
    let (raw_host_id, source) = read_host_id();
    let host_id = derive_host_id(&raw_host_id);

    let mut tags = std::collections::BTreeMap::new();
    tags.insert("host_id_source".to_string(), source);

    Ok(Host {
        host_id,
        install_id: install_id(state_dir)?,
        boot_id: read_trimmed("/proc/sys/kernel/random/boot_id").unwrap_or_default(),
        hostname: hostname(),
        fqdn: None,
        os: describe_os(),
        addresses: Vec::new(),
        tags,
        peer: None,
    })
}

fn read_host_id() -> (Vec<u8>, String) {
    for source in HOST_ID_SOURCES {
        if let Some(text) = read_trimmed(source)
            && !text.is_empty()
        {
            return (text.into_bytes(), (*source).to_string());
        }
    }
    (hostname().into_bytes(), "hostname".to_string())
}

fn derive_host_id(raw: &[u8]) -> String {
    hex(&hmac_sha256(raw, APPLICATION_ID)[..16])
}

fn hostname() -> String {
    read_trimmed("/proc/sys/kernel/hostname")
        .or_else(|| read_trimmed("/etc/hostname"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn describe_os() -> Os {
    let release = fs::read_to_string("/etc/os-release").unwrap_or_default();

    Os {
        family: std::env::consts::OS.to_string(),
        distro: os_release_field(&release, "ID").unwrap_or_else(|| "unknown".into()),
        version: os_release_field(&release, "VERSION_ID").unwrap_or_else(|| "unknown".into()),
        kernel: read_trimmed("/proc/sys/kernel/osrelease").unwrap_or_default(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn os_release_field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value.trim_matches('"').to_string())
}

fn read_trimmed(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
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
    fn reads_the_fields_it_needs_out_of_os_release_and_ignores_the_rest() {
        let text = "PRETTY_NAME=\"Alpine Linux v3.22\"\nID=alpine\nVERSION_ID=3.22.2\nHOME_URL=\"https://alpinelinux.org/\"\n";

        assert_eq!(os_release_field(text, "ID").as_deref(), Some("alpine"));
        assert_eq!(
            os_release_field(text, "VERSION_ID").as_deref(),
            Some("3.22.2")
        );
        assert_eq!(os_release_field(text, "NOPE"), None);
    }
}
