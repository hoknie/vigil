pub struct Origin {
    pub from: &'static str,
    pub reason: &'static str,
    pub dropped: &'static str,
    pub writable_paths: &'static [&'static str],
}

const LINUX_WRITABLE_PATHS: &[&str] = &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"];

#[cfg_attr(target_os = "linux", allow(dead_code))]
const MACOS_WRITABLE_PATHS: &[&str] = &[
    "/tmp/",
    "/var/tmp/",
    "/private/tmp/",
    "/private/var/tmp/",
    "/private/var/folders/",
    "/Users/",
];

const DROPPED_BY_THE_AUDIT_PLUGIN: &str = "the audit plugin dropped the oldest events to stay under its spool size; launches from that window were never read";

pub const AUDIT_PLUGIN: Origin = Origin {
    from: "audit plugin",
    reason: "launches arrive through the plugin auditd starts",
    dropped: DROPPED_BY_THE_AUDIT_PLUGIN,
    writable_paths: LINUX_WRITABLE_PATHS,
};

pub const AUDIT_LOG: Origin = Origin {
    from: "audit log",
    reason: "no plugin is delivering: launches are read from the log file, one reading late, and a rotation between two readings takes what it held",
    dropped: DROPPED_BY_THE_AUDIT_PLUGIN,
    writable_paths: LINUX_WRITABLE_PATHS,
};

#[cfg_attr(target_os = "linux", allow(dead_code))]
pub const ESLOGGER: Origin = Origin {
    from: "eslogger",
    reason: "launches arrive from Endpoint Security, through the eslogger the launchd job vigil.launches keeps running",
    dropped: "the job that spools what eslogger prints dropped the oldest launches to stay under its spool size; launches from that window were never read",
    writable_paths: MACOS_WRITABLE_PATHS,
};

impl Origin {
    pub fn writable(&self, executable: &str) -> bool {
        self.writable_paths
            .iter()
            .any(|writable| executable.starts_with(writable))
    }
}
