use std::fs;
use std::path::{Path, PathBuf};

const RUN_BY_SYSTEMD: [&str; 4] = ["ExecStart", "ExecStartPre", "ExecStartPost", "ExecStop"];

const PREFIXES: [char; 5] = ['-', '+', '!', '@', ':'];

fn units() -> Vec<(String, String)> {
    let directory = workspace().join("packaging/systemd");
    let mut read = Vec::new();

    for entry in
        fs::read_dir(&directory).unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
    {
        let path = entry.expect("a directory entry").path();
        let named = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if !named.ends_with(".service") && !named.ends_with(".timer") {
            continue;
        }
        read.push((named, fs::read_to_string(&path).expect("a unit file")));
    }

    assert!(
        !read.is_empty(),
        "no unit was read, so this guard reads nothing"
    );
    read
}

fn settings(unit: &str, name: &str) -> Vec<String> {
    unit.lines()
        .map(str::trim)
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| *key == name)
        .map(|(_, value)| value.trim().to_string())
        .collect()
}

#[test]
fn the_unit_names_the_absolute_path_of_every_binary_it_runs() {
    let mut commands = 0;

    for (named, unit) in units() {
        for setting in RUN_BY_SYSTEMD {
            for command in settings(&unit, setting) {
                let program = command
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_start_matches(PREFIXES);

                assert!(
                    program.starts_with('/'),
                    "{named} runs {program:?} by {setting}. A bare name is looked up in PATH, \
                     and what PATH means for a unit is not what it means in the shell the \
                     operator tested it in"
                );
                assert!(
                    Path::new(program).is_absolute(),
                    "{named}: {program:?} is not an absolute path"
                );
                commands += 1;
            }
        }
    }

    assert!(commands >= 2, "only {commands} command(s) were checked");
}

#[test]
fn nothing_the_firewall_reading_runs_takes_an_argument_from_the_host() {
    let unit = named("vigil-firewall.service");
    let started = settings(&unit, "ExecStart");

    assert_eq!(
        started,
        vec!["/usr/sbin/nft --json list ruleset".to_string()],
        "the command is three fixed words after an absolute path. A path, an address or a \
         table name taken from the host would be this product handing a value it read on the \
         host to a program it runs on the host"
    );
    assert!(
        unit.contains("ConditionPathExists=/usr/sbin/nft"),
        "with nft missing the unit must not run at all, so the agent reports a reading it \
         never got rather than a host with no rules"
    );
    assert!(
        settings(&unit, "TimeoutStartSec")
            .first()
            .is_some_and(|value| !value.eq_ignore_ascii_case("infinity")),
        "a run that hangs must end, or the reading ages while a process sits on the host"
    );
}

#[test]
fn the_timer_writes_the_reading_at_the_period_the_collector_reads_it() {
    let timer = named("vigil-firewall.timer");
    let declared = vigil_collect::every_seconds_of_collector("firewall")
        .expect("firewall is a collector this build ships");

    assert_eq!(
        settings(&timer, "OnUnitActiveSec"),
        vec![format!("{declared}s")],
        "the timer and the collector are two halves of one reading: a timer slower than the \
         collector makes every reading stale, and a faster one is work nobody looks at"
    );
    assert_eq!(
        settings(&timer, "Unit"),
        vec!["vigil-firewall.service".to_string()]
    );
}

#[test]
fn the_privileged_half_of_the_firewall_reading_is_not_in_the_daemons_unit() {
    let daemon = named("vigild.service");
    let reading = named("vigil-firewall.service");

    assert!(
        !daemon.contains("CAP_NET_ADMIN"),
        "the daemon runs all the time; the unit that reads the ruleset runs for half a second, \
         and that is where the capability belongs"
    );
    assert!(
        !daemon.contains("AF_NETLINK"),
        "vigild talks to a unix socket and to its reporters, and to nothing in the kernel's \
         netlink"
    );
    assert!(reading.contains("CAP_NET_ADMIN"), "{reading}");
    assert!(
        daemon.contains("Wants=vigil-firewall.timer"),
        "enabling the agent must start the reading, or the collector is unavailable on every \
         host nobody read the packaging notes on"
    );
}

#[test]
fn the_file_the_reading_is_written_to_is_the_one_the_collector_opens() {
    let unit = named("vigil-firewall.service");

    assert_eq!(
        settings(&unit, "StandardOutput"),
        vec!["truncate:/var/lib/vigil/firewall/ruleset.json".to_string()],
        "one path, written in the unit and opened by the collector; truncate keeps a reader \
         from seeing half of one ruleset and half of another"
    );
    assert!(
        fs::read_to_string(workspace().join("packaging/systemd/vigil-tmpfiles.conf"))
            .expect("the tmpfiles fragment")
            .contains("/var/lib/vigil/firewall"),
        "the directory the unit writes into is created by the packaging or the first run \
         fails with nothing to say"
    );
}

fn named(unit: &str) -> String {
    units()
        .into_iter()
        .find(|(name, _)| name == unit)
        .unwrap_or_else(|| panic!("{unit} is not in packaging/systemd"))
        .1
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root")
}
