use crate::parsers::{Domain, Scope};

pub(super) const LAUNCHD_DIRECTORIES: &[(&str, Domain, Scope)] = &[
    ("/Library/LaunchDaemons", Domain::Daemon, Scope::System),
    ("/Library/LaunchAgents", Domain::Agent, Scope::System),
    (
        "/System/Library/LaunchDaemons",
        Domain::Daemon,
        Scope::Vendor,
    ),
    ("/System/Library/LaunchAgents", Domain::Agent, Scope::Vendor),
];

pub(super) const HOMES: &str = "/Users";

pub(super) const NOT_A_PERSON: &[&str] = &["Shared", "Guest"];

pub(super) const PERSONAL_AGENTS: &str = "Library/LaunchAgents";

pub(super) const CRONTAB: &str = "/etc/crontab";

pub(super) const CRON_TABLES: &str = "/usr/lib/cron/tabs";

pub(super) const PERIODIC_DIRECTORIES: &[(&str, &str)] = &[
    ("/etc/periodic/daily", "@daily"),
    ("/etc/periodic/weekly", "@weekly"),
    ("/etc/periodic/monthly", "@monthly"),
    ("/usr/local/etc/periodic/daily", "@daily"),
    ("/usr/local/etc/periodic/weekly", "@weekly"),
    ("/usr/local/etc/periodic/monthly", "@monthly"),
];

pub(super) const SYSTEM_PROFILES: &[&str] = &[
    "/etc/profile",
    "/etc/bashrc",
    "/etc/zshenv",
    "/etc/zprofile",
    "/etc/zshrc",
    "/etc/zlogin",
];

pub(super) const PERSONAL_PROFILES: &[&str] = &[
    ".zshenv",
    ".zprofile",
    ".zshrc",
    ".zlogin",
    ".bash_profile",
    ".bashrc",
    ".profile",
];

pub(super) const LOGIN_HOOKS: &str = "/var/root/Library/Preferences/com.apple.loginwindow.plist";

pub(super) const HOOKS: &[&str] = &["LoginHook", "LogoutHook"];

pub(super) const FILE_LIMIT: u64 = 1024 * 1024;

pub(super) const DIRECTORY_LIMIT: usize = 4096;
