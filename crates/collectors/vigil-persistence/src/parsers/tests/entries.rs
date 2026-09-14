use super::super::crontab::CronEntry;
use super::super::crontab::{CronFormat, parse_crontab};
use super::super::entries::MODULES_UNREADABLE;
use super::super::entries::{
    PersistenceReading, PreloadFile, ScriptFamily, UnitFile, WatchedScript, persistence_snapshot,
};
use super::super::unit::parse_unit;
use serde_json::json;

fn preload_absent() -> PreloadFile {
    PreloadFile {
        path: "/etc/ld.so.preload".into(),
        present: false,
        readable: true,
        entries: Vec::new(),
        digest: None,
    }
}

fn reading<'a>(
    units: &'a [UnitFile],
    cron: &'a [CronEntry],
    preload: &'a PreloadFile,
) -> PersistenceReading<'a> {
    PersistenceReading {
        units,
        cron,
        modules: Some(&[]),
        scripts: &[],
        preload,
    }
}

fn script(path: &str, present: Option<bool>, shown: bool) -> WatchedScript {
    WatchedScript {
        path: path.into(),
        family: ScriptFamily::Profile,
        present,
        shown,
        readable: shown.then_some(true),
        digest: None,
        size: 0,
        mode: String::new(),
        uid: 0,
        gid: 0,
    }
}

#[test]
fn a_file_the_agent_was_not_shown_is_not_a_file_that_is_no_longer_there() {
    let preload = preload_absent();
    let scripts = [
        script("/tmp/.hidden/.bashrc", None, false),
        script("/etc/profile", Some(false), true),
    ];
    let mut reading = reading(&[], &[], &preload);
    reading.scripts = &scripts;

    let snapshot = persistence_snapshot("2026-09-09T12:00:00.000Z", &reading);

    let unseen = &snapshot.items["script|/tmp/.hidden/.bashrc"];
    assert_eq!(unseen["present"], json!(null));
    assert_eq!(unseen["shown"], json!(false));
    assert_eq!(
        unseen["readable"],
        json!(null),
        "a path the agent has its own copy of answers nothing about the host's"
    );

    let gone = &snapshot.items["script|/etc/profile"];
    assert_eq!(gone["present"], json!(false));
    assert_eq!(gone["shown"], json!(true));
}

#[test]
fn a_timer_and_a_service_are_different_kinds_of_key() {
    let units = vec![
        UnitFile {
            name: "nginx.service".into(),
            path: "/lib/systemd/system/nginx.service".into(),
            readable: true,
            facts: parse_unit("[Service]\nExecStart=/usr/sbin/nginx\n"),
        },
        UnitFile {
            name: "certbot.timer".into(),
            path: "/lib/systemd/system/certbot.timer".into(),
            readable: true,
            facts: parse_unit("[Timer]\nOnCalendar=daily\n"),
        },
    ];

    let preload = preload_absent();
    let snapshot =
        persistence_snapshot("2026-09-09T12:00:00.000Z", &reading(&units, &[], &preload));

    assert_eq!(snapshot.source, "persistence");
    assert_eq!(snapshot.items["unit|nginx.service"]["type"], "service");
    assert_eq!(snapshot.items["unit|nginx.service"]["run_as"], "root");
    assert_eq!(
        snapshot.items["timer|certbot.timer"]["activates"], "certbot.service",
        "a timer with no Unit= starts the service of the same name"
    );
    assert!(
        !snapshot.items.contains_key("unit|certbot.timer"),
        "a timer must not also be a unit, or one file is two findings"
    );
}

#[test]
fn a_unit_carries_what_the_file_says_pulls_it_in_and_what_it_pulls() {
    let units = vec![UnitFile {
        name: "nginx.service".into(),
        path: "/lib/systemd/system/nginx.service".into(),
        readable: true,
        facts: parse_unit(
            "[Unit]\nWants=network-online.target\nAfter=network.target\n\
             [Service]\nExecStart=/usr/sbin/nginx\n\
             [Install]\nWantedBy=multi-user.target\n",
        ),
    }];

    let preload = preload_absent();
    let snapshot =
        persistence_snapshot("2026-09-09T12:00:00.000Z", &reading(&units, &[], &preload));

    let unit = &snapshot.items["unit|nginx.service"];
    assert_eq!(unit["wanted_by"], json!(["multi-user.target"]));
    assert_eq!(unit["wants"], json!(["network-online.target"]));
    for absent in ["required_by", "requires", "part_of"] {
        assert_eq!(
            unit.get(absent),
            None,
            "a setting the file does not carry costs nothing in the snapshot"
        );
    }
    assert_eq!(
        unit.get("after"),
        None,
        "the order units start in is not what pulls them in, and is not recorded as if \
         it were"
    );
}

#[test]
fn a_cron_key_is_the_file_the_user_and_the_command() {
    let jobs = parse_crontab(
        "@reboot /tmp/.x/implant\n",
        "/var/spool/cron/crontabs/www-data",
        CronFormat::ForOneUser,
        "www-data",
    );

    let preload = preload_absent();
    let snapshot = persistence_snapshot("2026-09-09T12:00:00.000Z", &reading(&[], &jobs, &preload));

    assert!(
        snapshot
            .items
            .contains_key("cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant"),
        "{:?}",
        snapshot.items.keys().collect::<Vec<_>>()
    );
}

#[test]
fn a_kernel_whose_module_list_we_could_not_read_says_so_instead_of_looking_empty() {
    let snapshot = persistence_snapshot(
        "2026-09-09T12:00:00.000Z",
        &PersistenceReading {
            units: &[],
            cron: &[],
            modules: None,
            scripts: &[],
            preload: &preload_absent(),
        },
    );

    assert_eq!(snapshot.items[MODULES_UNREADABLE]["readable"], false);
    assert!(
        !snapshot.items.keys().any(|key| key.starts_with("module|")),
        "the marker must not look like a module"
    );
}

#[test]
fn the_preload_file_is_an_item_even_when_the_host_does_not_have_one() {
    let preload = preload_absent();
    let snapshot = persistence_snapshot("2026-09-09T12:00:00.000Z", &reading(&[], &[], &preload));

    assert_eq!(
        snapshot.items["preload|/etc/ld.so.preload"]["present"],
        false
    );
}
