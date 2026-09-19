use vigil_model::Snapshot;

use crate::parsers::{
    CronFormat, Domain, LaunchdJob, MacosPersistenceReading, Scope, ScriptFamily, WatchedScript,
    cron_script, launchd_facts, macos_persistence_snapshot, parse_crontab, parse_plist,
};

const UPDATER: &[u8] = include_bytes!("../parsers/plist/tests/job.xml");

const UPDATER_AS_BINARY: &[u8] = include_bytes!("../parsers/plist/tests/job.binary");

const AGENT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>com.example.sync</string>
	<key>ProgramArguments</key>
	<array>
		<string>/Users/alice/Library/Application Support/Sync/sync</string>
		<string>--background</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
</dict>
</plist>
"#;

const VENDOR: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>com.apple.example</string>
	<key>Program</key>
	<string>/usr/libexec/example</string>
	<key>MachServices</key>
	<dict>
		<key>com.apple.example</key>
		<true/>
	</dict>
</dict>
</plist>
"#;

const CRONTAB: &str = "*/10 * * * * /Users/alice/bin/backup --token=abc123\n";

fn job(path: &str, bytes: &[u8], domain: Domain, scope: Scope, owner: Option<&str>) -> LaunchdJob {
    LaunchdJob {
        path: path.to_string(),
        domain,
        scope,
        owner: owner.map(str::to_string),
        readable: true,
        facts: parse_plist(bytes)
            .map(|value| launchd_facts(&value))
            .map_err(|refusal| refusal.to_string()),
    }
}

fn jobs() -> Vec<LaunchdJob> {
    vec![
        job(
            "/Library/LaunchDaemons/com.example.updater.plist",
            UPDATER,
            Domain::Daemon,
            Scope::System,
            None,
        ),
        job(
            "/Library/LaunchAgents/com.example.updater.agent.plist",
            UPDATER_AS_BINARY,
            Domain::Agent,
            Scope::System,
            None,
        ),
        job(
            "/System/Library/LaunchDaemons/com.apple.example.plist",
            VENDOR.as_bytes(),
            Domain::Daemon,
            Scope::Vendor,
            None,
        ),
        job(
            "/Users/alice/Library/LaunchAgents/com.example.sync.plist",
            AGENT.as_bytes(),
            Domain::Agent,
            Scope::Person,
            Some("alice"),
        ),
        job(
            "/Users/alice/Library/LaunchAgents/notes.plist",
            b"remember to renew the certificate",
            Domain::Agent,
            Scope::Person,
            Some("alice"),
        ),
        LaunchdJob {
            path: "/Library/LaunchDaemons/com.example.locked.plist".to_string(),
            domain: Domain::Daemon,
            scope: Scope::System,
            owner: None,
            readable: false,
            facts: Err("the file cannot be read: Permission denied (os error 13)".to_string()),
        },
    ]
}

fn script(path: &str, family: ScriptFamily, readable: bool) -> WatchedScript {
    WatchedScript {
        path: path.to_string(),
        family,
        present: readable.then_some(true),
        shown: true,
        readable: Some(readable),
        digest: readable.then(|| "3f".repeat(32)),
        size: match readable {
            true => 3191,
            false => 0,
        },
        mode: match readable {
            true => "0444".to_string(),
            false => String::new(),
        },
        uid: 0,
        gid: 0,
    }
}

pub fn persistence_on_macos() -> Snapshot {
    let mut cron = parse_crontab(
        CRONTAB,
        "/usr/lib/cron/tabs/alice",
        CronFormat::ForOneUser,
        "alice",
    );
    cron.push(cron_script(
        "/usr/local/etc/periodic/daily",
        "/usr/local/etc/periodic/daily/500.backup",
        "@daily",
    ));

    macos_persistence_snapshot(
        "2026-09-19T09:00:00.000Z",
        &MacosPersistenceReading {
            jobs: &jobs(),
            cron: &cron,
            scripts: &[
                script("/etc/zshrc", ScriptFamily::Profile, true),
                script("/Users/bob/.zshrc", ScriptFamily::Profile, false),
                script("/Library/Scripts/login.sh", ScriptFamily::Boot, true),
            ],
        },
    )
}
