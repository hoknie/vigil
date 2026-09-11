use vigil_model::Snapshot;

use crate::parsers::{
    CronFormat, PersistenceReading, PreloadFile, ScriptFamily, UnitFile, WatchedScript,
    cron_script, parse_crontab, parse_modules, parse_unit, persistence_snapshot,
};

const MODULES_UNREADABLE: &str = "modules|unreadable";

const NGINX: &str = "\
[Unit]
Description=A high performance web server
Wants=nss-lookup.target
Requires=network.target
PartOf=web.target

[Service]
ExecStartPre=/usr/sbin/nginx -t
ExecStart=/usr/sbin/nginx -g 'daemon off;' --password hunter2
User=www-data

[Install]
WantedBy=multi-user.target
RequiredBy=web.target
";

const TARGET: &str = "\
[Unit]
Description=Multi-User System
";

const RESCUE: &str = "\
[Service]
ExecStart=/bin/sh
";

const LOGROTATE: &str = "\
[Unit]
Description=Daily rotation of log files

[Timer]
OnCalendar=daily
OnCalendar=Mon *-*-* 03:15:00
Unit=logrotate.service
";

const WATCHDOG: &str = "\
[Timer]
OnBootSec=15min
";

const CRONTAB: &str = "\
17 *	* * *	root	cd / && run-parts --report /etc/cron.hourly
@daily root /usr/local/bin/backup --to /srv
";

const SPOOL: &str = "*/5 * * * * /tmp/.x/implant\n";

const MODULES: &str = "\
overlay 155648 1 - Live 0xffffffffc0a00000
nf_nat 49152 2 xt_MASQUERADE,nft_chain_nat - Live 0xffffffffc0800000
";

fn unit(name: &str, path: &str, text: &str, readable: bool) -> UnitFile {
    UnitFile {
        name: name.to_string(),
        path: path.to_string(),
        readable,
        facts: parse_unit(text),
    }
}

fn units() -> Vec<UnitFile> {
    vec![
        unit(
            "nginx.service",
            "/lib/systemd/system/nginx.service",
            NGINX,
            true,
        ),
        unit(
            "multi-user.target",
            "/lib/systemd/system/multi-user.target",
            TARGET,
            true,
        ),
        unit(
            "rescue-shell.service",
            "/etc/systemd/system/rescue-shell.service",
            RESCUE,
            true,
        ),
        unit(
            "locked.service",
            "/etc/systemd/system/locked.service",
            "",
            false,
        ),
        unit(
            "logrotate.timer",
            "/lib/systemd/system/logrotate.timer",
            LOGROTATE,
            true,
        ),
        unit(
            "watchdog.timer",
            "/etc/systemd/system/watchdog.timer",
            WATCHDOG,
            true,
        ),
    ]
}

fn scripts() -> Vec<WatchedScript> {
    vec![
        WatchedScript {
            path: "/etc/profile".into(),
            family: ScriptFamily::Profile,
            present: Some(true),
            shown: true,
            readable: Some(true),
            digest: Some("9f2c1b3d4e5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c".into()),
            size: 581,
            mode: "0644".into(),
            uid: 0,
            gid: 0,
        },
        WatchedScript {
            path: "/etc/rc.local".into(),
            family: ScriptFamily::Boot,
            present: Some(false),
            shown: true,
            readable: Some(true),
            digest: None,
            size: 0,
            mode: String::new(),
            uid: 0,
            gid: 0,
        },
        WatchedScript {
            path: "/tmp/.hidden/.bashrc".into(),
            family: ScriptFamily::Profile,
            present: None,
            shown: false,
            readable: None,
            digest: None,
            size: 0,
            mode: String::new(),
            uid: 0,
            gid: 0,
        },
    ]
}

fn preload() -> PreloadFile {
    PreloadFile {
        path: "/etc/ld.so.preload".into(),
        present: true,
        readable: true,
        entries: vec!["/usr/local/lib/libjackit.so".into()],
        digest: Some("1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b".into()),
    }
}

fn reading<'a>(
    units: &'a [UnitFile],
    cron: &'a [crate::parsers::CronEntry],
    modules: Option<&'a [crate::parsers::KernelModule]>,
    scripts: &'a [WatchedScript],
    preload: &'a PreloadFile,
) -> PersistenceReading<'a> {
    PersistenceReading {
        units,
        cron,
        modules,
        scripts,
        preload,
    }
}

pub fn persistence() -> Snapshot {
    let units = units();
    let mut cron = parse_crontab(CRONTAB, "/etc/crontab", CronFormat::WithUser, "root");
    cron.extend(parse_crontab(
        SPOOL,
        "/var/spool/cron/crontabs/www-data",
        CronFormat::ForOneUser,
        "www-data",
    ));
    cron.push(cron_script(
        "/etc/cron.daily",
        "/etc/cron.daily/logrotate",
        "@daily",
    ));
    let modules = parse_modules(MODULES);
    let scripts = scripts();
    let preload = preload();

    let mut snapshot = persistence_snapshot(
        "2026-09-09T09:00:00.000Z",
        &reading(&units, &cron, Some(&modules), &scripts, &preload),
    );

    let unreadable = persistence_snapshot(
        "2026-09-09T09:00:00.000Z",
        &reading(&units, &cron, None, &scripts, &preload),
    );
    if let Some(note) = unreadable.items.get(MODULES_UNREADABLE) {
        snapshot
            .items
            .insert(MODULES_UNREADABLE.to_string(), note.clone());
    }

    snapshot
}
