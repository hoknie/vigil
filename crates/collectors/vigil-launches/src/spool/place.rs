use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const SPOOL_PATH: &str = "/var/lib/vigil/audit-spool";

pub const PLUGIN_CONFIG_PATH: &str = "/etc/audit/plugins.d/vigil.conf";

pub const CEILING_BYTES: u64 = 16 * 1024 * 1024;

pub fn cursor_path(spool: &Path) -> PathBuf {
    beside(spool, ".cursor")
}

pub fn writing_path(spool: &Path) -> PathBuf {
    beside(spool, ".writing")
}

fn beside(path: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(suffix);
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cursor_and_the_temporary_sit_beside_the_spool_they_belong_to() {
        let spool = Path::new("/var/lib/vigil/audit-spool");

        assert_eq!(
            cursor_path(spool),
            PathBuf::from("/var/lib/vigil/audit-spool.cursor")
        );
        assert_eq!(
            writing_path(spool),
            PathBuf::from("/var/lib/vigil/audit-spool.writing")
        );
    }

    #[test]
    fn a_spool_with_an_extension_keeps_it() {
        assert_eq!(
            cursor_path(Path::new("/tmp/recorded.log")),
            PathBuf::from("/tmp/recorded.log.cursor")
        );
    }
}
