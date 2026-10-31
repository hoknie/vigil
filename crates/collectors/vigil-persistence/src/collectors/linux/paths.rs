pub(in super::super) const UNIT_DIRECTORIES: &[&str] = &[
    "/etc/systemd/system",
    "/run/systemd/system",
    "/usr/local/lib/systemd/system",
    "/usr/lib/systemd/system",
    "/lib/systemd/system",
];

pub(in super::super) const CRONTAB: &str = "/etc/crontab";

pub(in super::super) const CRON_DIRECTORY: &str = "/etc/cron.d";

pub(in super::super) const CRON_SPOOLS: &[&str] = &[
    "/var/spool/cron/crontabs",
    "/var/spool/cron",
    "/var/spool/cron/tabs",
    "/etc/crontabs",
];

pub(in super::super) const DEBIAN_VERSION: &str = "/etc/debian_version";

pub(in super::super) const CRON_SCRIPT_DIRECTORIES: &[(&str, &str)] = &[
    ("/etc/cron.hourly", "@hourly"),
    ("/etc/cron.daily", "@daily"),
    ("/etc/cron.weekly", "@weekly"),
    ("/etc/cron.monthly", "@monthly"),
    ("/etc/periodic/15min", "*/15 * * * *"),
    ("/etc/periodic/hourly", "@hourly"),
    ("/etc/periodic/daily", "@daily"),
    ("/etc/periodic/weekly", "@weekly"),
    ("/etc/periodic/monthly", "@monthly"),
];

pub(in super::super) const PRELOAD: &str = "/etc/ld.so.preload";

pub(in super::super) const BOOT_SCRIPTS: &[&str] = &["/etc/rc.local", "/etc/rc.d/rc.local"];

pub(in super::super) const BOOT_DIRECTORIES: &[&str] = &["/etc/local.d"];

pub(in super::super) const SYSTEM_PROFILES: &[&str] = &[
    "/etc/profile",
    "/etc/bash.bashrc",
    "/etc/bashrc",
    "/etc/zsh/zshrc",
    "/etc/zsh/zprofile",
    "/etc/zshrc",
    "/etc/environment",
    "/usr/etc/profile",
    "/usr/etc/bash.bashrc",
    "/usr/etc/environment",
];

pub(in super::super) const PROFILE_DIRECTORIES: &[&str] = &["/etc/profile.d", "/usr/etc/profile.d"];

pub(in super::super) const USER_PROFILES: &[&str] = &[
    ".bashrc",
    ".bash_profile",
    ".bash_login",
    ".bash_logout",
    ".profile",
    ".zshrc",
    ".zprofile",
];
