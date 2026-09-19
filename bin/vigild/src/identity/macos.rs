use std::ffi::CString;

use vigil_model::Os;

pub const HOST_ID_SOURCE: &str = "gethostuuid";

const DISTRIBUTION: &str = "macos";

const UUID_BYTES: usize = 16;

const LONGEST_ANSWER: usize = 256;

pub fn host_id() -> (Vec<u8>, String) {
    match host_uuid() {
        Some(uuid) => (uuid.into_bytes(), HOST_ID_SOURCE.to_string()),
        None => (hostname().into_bytes(), "hostname".to_string()),
    }
}

pub fn boot_id() -> String {
    said("kern.bootsessionuuid").unwrap_or_default()
}

pub fn hostname() -> String {
    said("kern.hostname").unwrap_or_else(|| "unknown".to_string())
}

pub fn os() -> Os {
    Os {
        family: std::env::consts::OS.to_string(),
        distro: DISTRIBUTION.to_string(),
        version: said("kern.osproductversion").unwrap_or_else(|| "unknown".into()),
        kernel: said("kern.osrelease").unwrap_or_default(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn host_uuid() -> Option<String> {
    let mut uuid = [0u8; UUID_BYTES];
    let wait = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let answered = unsafe { libc::gethostuuid(uuid.as_mut_ptr(), &wait) };
    match answered == 0 && uuid.iter().any(|byte| *byte != 0) {
        true => Some(spelled(&uuid)),
        false => None,
    }
}

fn spelled(uuid: &[u8; UUID_BYTES]) -> String {
    let hex: Vec<String> = uuid.iter().map(|byte| format!("{byte:02X}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        hex[0..4].concat(),
        hex[4..6].concat(),
        hex[6..8].concat(),
        hex[8..10].concat(),
        hex[10..16].concat()
    )
}

fn said(name: &str) -> Option<String> {
    let name = CString::new(name).ok()?;
    let mut answer = [0u8; LONGEST_ANSWER];
    let mut length: libc::size_t = answer.len();
    let answered = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            answer.as_mut_ptr().cast(),
            &mut length,
            std::ptr::null_mut(),
            0,
        )
    };
    if answered != 0 {
        return None;
    }
    let text = &answer[..length.min(answer.len())];
    let text = text.split(|byte| *byte == 0).next().unwrap_or_default();
    let text = String::from_utf8_lossy(text).trim().to_string();
    match text.is_empty() {
        true => None,
        false => Some(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_host_is_named_by_the_hardware_uuid_the_platform_expert_keeps_and_not_by_its_name() {
        let (raw, source) = host_id();

        assert_eq!(
            source, HOST_ID_SOURCE,
            "a Mac has no /etc/machine-id; its stable identity is IOPlatformUUID, and a \
             hostname changes with the network the laptop joins"
        );
        assert_eq!(raw.len(), 36, "{}", String::from_utf8_lossy(&raw));
    }

    #[test]
    fn a_uuid_is_spelled_the_way_system_information_prints_it() {
        let uuid = [
            0x17, 0xE0, 0xBC, 0xD5, 0xCA, 0xBB, 0x43, 0x6C, 0x90, 0xA9, 0x99, 0x72, 0x68, 0x18,
            0xCE, 0x57,
        ];

        assert_eq!(spelled(&uuid), "17E0BCD5-CABB-436C-90A9-99726818CE57");
    }

    #[test]
    fn the_boot_the_name_and_the_release_are_read_from_the_kernel_of_this_mac() {
        assert_eq!(boot_id().len(), 36, "kern.bootsessionuuid");
        assert_ne!(hostname(), "unknown");
        let os = os();
        assert_eq!(os.distro, "macos");
        assert_ne!(os.version, "unknown", "kern.osproductversion");
        assert!(!os.kernel.is_empty(), "kern.osrelease");
    }

    #[test]
    fn a_name_the_kernel_does_not_know_is_nothing_rather_than_an_empty_word() {
        assert_eq!(said("kern.vigil-no-such-name"), None);
    }
}
