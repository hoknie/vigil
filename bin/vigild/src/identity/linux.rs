use std::fs;

use vigil_model::Os;

use super::host::HOST_ID_SOURCES;

pub fn host_id() -> (Vec<u8>, String) {
    for source in HOST_ID_SOURCES {
        if let Some(text) = read_trimmed(source)
            && !text.is_empty()
        {
            return (text.into_bytes(), (*source).to_string());
        }
    }
    (hostname().into_bytes(), "hostname".to_string())
}

pub fn boot_id() -> String {
    read_trimmed("/proc/sys/kernel/random/boot_id").unwrap_or_default()
}

pub fn hostname() -> String {
    read_trimmed("/proc/sys/kernel/hostname")
        .or_else(|| read_trimmed("/etc/hostname"))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn os() -> Os {
    let release = OS_RELEASE
        .iter()
        .find_map(|place| fs::read_to_string(place).ok())
        .unwrap_or_default();

    Os {
        family: std::env::consts::OS.to_string(),
        distro: os_release_field(&release, "ID").unwrap_or_else(|| "unknown".into()),
        version: version_of(&release),
        kernel: read_trimmed("/proc/sys/kernel/osrelease").unwrap_or_default(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

const OS_RELEASE: &[&str] = &["/etc/os-release", "/usr/lib/os-release"];

fn version_of(release: &str) -> String {
    os_release_field(release, "VERSION_ID")
        .or_else(|| os_release_field(release, "BUILD_ID"))
        .unwrap_or_else(|| "unknown".into())
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
    fn reads_the_fields_it_needs_out_of_os_release_and_ignores_the_rest() {
        let text = "PRETTY_NAME=\"Alpine Linux v3.22\"\nID=alpine\nVERSION_ID=3.22.2\nHOME_URL=\"https://alpinelinux.org/\"\n";

        assert_eq!(os_release_field(text, "ID").as_deref(), Some("alpine"));
        assert_eq!(
            os_release_field(text, "VERSION_ID").as_deref(),
            Some("3.22.2")
        );
        assert_eq!(os_release_field(text, "NOPE"), None);
    }

    #[test]
    fn a_rolling_distribution_with_no_version_is_named_by_its_build() {
        let arch = "NAME=\"Arch Linux\"\nID=arch\nBUILD_ID=rolling\n";

        assert_eq!(version_of(arch), "rolling");
        assert_eq!(
            version_of("ID=debian\nVERSION_ID=\"12\"\nBUILD_ID=ignored\n"),
            "12",
            "a version, when there is one, is what the summary shows"
        );
        assert_eq!(version_of("ID=gentoo\n"), "unknown");
    }

    #[test]
    fn the_description_of_the_system_is_looked_for_where_the_os_release_standard_puts_it() {
        assert_eq!(
            OS_RELEASE,
            ["/etc/os-release", "/usr/lib/os-release"],
            "a host that ships only the file under /usr/lib, as some minimal images do, is \
             still a named system and not an unknown one"
        );
    }
}
