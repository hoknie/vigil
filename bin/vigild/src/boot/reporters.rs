use std::path::Path;

use vigil_report::{NdjsonSink, Reporter, SyslogFacility, SyslogSink};
use vigil_store::{create_owner_only, create_owner_only_directory};

use crate::Receiver;

pub fn build(receivers: &[Receiver]) -> Result<Vec<Box<dyn Reporter>>, String> {
    let mut reporters: Vec<Box<dyn Reporter>> = Vec::new();

    for receiver in receivers {
        match receiver {
            Receiver::Ndjson { path } => {
                let file = open_owner_only(Path::new(path))
                    .map_err(|error| format!("the ndjson reporter cannot open {path}: {error}"))?;
                reporters.push(Box::new(NdjsonSink::new("ndjson", file)));
            }
            Receiver::Syslog { facility } => {
                let facility = SyslogFacility::parse(facility)
                    .map_err(|error| format!("the syslog reporter: {error}"))?;
                reporters.push(Box::new(
                    SyslogSink::open("syslog", vigil_report::LOCAL_SOCKET, facility, "vigil")
                        .map_err(|error| format!("the syslog reporter cannot connect: {error}"))?,
                ));
            }
            Receiver::Webhook { .. } => {
                return Err(
                    "the webhook reporter is configured but not implemented in this build".into(),
                );
            }
            Receiver::HostFindings { .. } => {
                return Err(
                    "the host-findings reporter is configured but not implemented in this build"
                        .into(),
                );
            }
        }
    }

    Ok(reporters)
}

fn open_owner_only(path: &Path) -> Result<std::fs::File, vigil_store::StoreError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        create_owner_only_directory(parent)?;
    }
    create_owner_only(path, true)
}

pub fn names(reporters: &[Box<dyn Reporter>]) -> Vec<String> {
    reporters
        .iter()
        .map(|reporter| reporter.name().to_string())
        .collect()
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use super::*;

    fn temporary_directory(name: &str) -> PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "vigild-reporters-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos()
                    + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
                .unwrap_or(0)
        ))
    }

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).expect("stat").permissions().mode() & 0o777
    }

    fn ndjson_at(path: &Path) -> Vec<Receiver> {
        vec![Receiver::Ndjson {
            path: path.display().to_string(),
        }]
    }

    #[test]
    fn an_ndjson_file_is_created_owner_only_whatever_the_umask() {
        let top = temporary_directory("umask");
        let path = top.join("log").join("findings.ndjson");

        let previous = rustix::process::umask(rustix::fs::Mode::from_raw_mode(0o022));
        let built = build(&ndjson_at(&path));
        rustix::process::umask(previous);

        built.expect("builds");
        assert_eq!(
            mode(&path),
            0o600,
            "a daemon started by hand, not by its unit, writes findings nobody else may read"
        );
        assert_eq!(mode(&top.join("log")), 0o700);
        assert_eq!(mode(&top), 0o700);
        let _ = std::fs::remove_dir_all(&top);
    }

    #[test]
    fn an_ndjson_file_wider_than_owner_only_is_narrowed_at_start_and_keeps_its_lines() {
        let top = temporary_directory("narrowed");
        std::fs::create_dir_all(&top).expect("makes the directory");
        let path = top.join("findings.ndjson");
        std::fs::write(&path, "{\"line\":1}\n").expect("writes");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");

        build(&ndjson_at(&path)).expect("builds");

        assert_eq!(mode(&path), 0o600);
        assert_eq!(
            std::fs::read_to_string(&path).expect("readable"),
            "{\"line\":1}\n",
            "narrowing the file is not starting it again"
        );
        let _ = std::fs::remove_dir_all(&top);
    }

    #[test]
    fn a_directory_that_already_exists_keeps_its_mode() {
        let top = temporary_directory("kept");
        std::fs::create_dir_all(&top).expect("makes the directory");
        std::fs::set_permissions(&top, std::fs::Permissions::from_mode(0o755)).expect("chmod");

        build(&ndjson_at(&top.join("findings.ndjson"))).expect("builds");

        assert_eq!(
            mode(&top),
            0o755,
            "a directory like /var/log is the host's, not this agent's"
        );
        let _ = std::fs::remove_dir_all(&top);
    }
}
