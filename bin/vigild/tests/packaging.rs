use std::fs;
use std::path::{Path, PathBuf};

const RUN_BY_SYSTEMD: [&str; 4] = ["ExecStart", "ExecStartPre", "ExecStartPost", "ExecStop"];

const PREFIXES: [char; 5] = ['-', '+', '!', '@', ':'];

const ASKED_FOR: &str = "\"collector\":\"";

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
    let declared = vigil_module::Module::every_seconds(&vigil_firewall::Firewall);

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
fn nothing_the_container_reading_runs_takes_an_argument_from_the_host() {
    let unit = named("vigil-containers.service");
    let started = settings(&unit, "ExecStart");

    assert_eq!(
        started,
        vec!["/usr/sbin/vigil-container-dump --directory /var/lib/vigil/containers".to_string()],
        "the command is this package's own binary and one fixed path. The engine clients it \
         runs are named inside it, from a list a test reads, and no argument of any of them \
         comes off the host"
    );
    assert!(
        settings(&unit, "TimeoutStartSec")
            .first()
            .is_some_and(|value| !value.eq_ignore_ascii_case("infinity")),
        "a run that hangs must end, or the reading ages while a docker client sits on the host"
    );
    assert!(
        unit.contains("StateDirectory=vigil/containers") && unit.contains("UMask=0077"),
        "the directory the dump is written into holds the labels of every container on this \
         host, and a label holds whatever the person who wrote the compose file put in it"
    );
}

#[test]
fn the_timer_asks_the_engines_at_the_period_the_collector_reads_them() {
    let timer = named("vigil-containers.timer");
    let declared = vigil_module::Module::every_seconds(&vigil_engines::Engines);

    assert_eq!(
        settings(&timer, "OnUnitActiveSec"),
        vec![format!("{declared}s")],
        "the timer and the collector are two halves of one reading: a timer slower than the \
         collector makes every reading stale, and a faster one is work nobody looks at"
    );
    assert_eq!(
        settings(&timer, "Unit"),
        vec!["vigil-containers.service".to_string()]
    );
    assert_eq!(
        vigil_module::Module::unit(&vigil_engines::Engines),
        Some("vigil-containers.timer"),
        "`vigild collector containers-engines enable` enables whatever the module names here, \
         and a name that is not a unit of this package enables nothing"
    );
}

#[test]
fn the_directory_the_engines_are_written_into_is_created_by_the_packaging() {
    assert!(
        fs::read_to_string(workspace().join("packaging/systemd/vigil-tmpfiles.conf"))
            .expect("the tmpfiles fragment")
            .contains("/var/lib/vigil/containers"),
        "the directory the dump writes into is created by the packaging or the first run \
         fails with nothing to say"
    );
    let daemon = named("vigild.service");
    assert!(
        daemon.contains("Wants=vigil-containers.timer"),
        "enabling the agent must start the reading, or the collector is unavailable on every \
         host nobody read the packaging notes on"
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

fn shipped_configuration() -> Vec<String> {
    let directory = workspace().join("config/collectors");
    let mut collectors: Vec<String> = fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            format!(
                "/etc/vigil/collectors/{}",
                entry.expect("an entry").file_name().to_string_lossy()
            )
        })
        .collect();
    collectors.sort();
    assert!(!collectors.is_empty(), "config/collectors holds nothing");

    let mut every = vec!["/etc/vigil/vigil.yaml".to_string()];
    every.extend(collectors);
    every.push("/etc/vigil/watch_fs.yaml".to_string());
    every
}

fn shipped_collectors() -> Vec<String> {
    let directory = workspace().join("config/collectors");
    let mut names = Vec::new();
    for entry in fs::read_dir(&directory).expect("config/collectors") {
        let text = fs::read_to_string(entry.expect("an entry").path()).expect("a file");
        for document in serde_yaml::Deserializer::from_str(&text) {
            let value = <serde_yaml::Value as serde::Deserialize>::deserialize(document)
                .expect("a document");
            if let Some(mapping) = value.as_mapping() {
                names.extend(
                    mapping
                        .keys()
                        .filter_map(|key| key.as_str().map(str::to_string)),
                );
            }
        }
    }
    names
}

#[test]
fn every_shipped_configuration_file_is_a_conffile_of_both_packages() {
    let debian =
        fs::read_to_string(workspace().join("packaging/deb/conffiles")).expect("conffiles");
    let listed: Vec<&str> = debian
        .lines()
        .filter(|line| line.starts_with("/etc/vigil/"))
        .collect();
    let spec = fs::read_to_string(workspace().join("packaging/rpm/vigil.spec")).expect("the spec");
    let noreplace: Vec<&str> = spec
        .lines()
        .filter_map(|line| line.strip_prefix("%config(noreplace) %attr(0600,root,root) "))
        .filter(|path| path.starts_with("/etc/vigil/"))
        .collect();

    assert_eq!(
        listed,
        shipped_configuration(),
        "a file under config/ that is not a conffile is overwritten by the next upgrade, and \
         the operator's edit with it"
    );
    assert_eq!(noreplace, shipped_configuration());
    assert!(
        spec.contains("%dir %attr(0700,root,root) /etc/vigil/collectors"),
        "the collectors directory may name a file with a token in it, so it is root's alone"
    );
}

#[test]
fn the_package_installs_every_file_config_ships_owner_only() {
    let script =
        fs::read_to_string(workspace().join("env/scripts/package.sh")).expect("package.sh");

    for line in [
        "install -d -m 0700 \"$tree/etc/vigil/collectors\"",
        "for collectors in \"$ROOT\"/config/collectors/*.yaml; do",
        "install -m 0600 \"$collectors\" \"$tree/etc/vigil/collectors/$(basename \"$collectors\")\"",
        "install -m 0600 \"$ROOT/config/watch_fs.yaml\" \"$tree/etc/vigil/watch_fs.yaml\"",
    ] {
        assert!(script.contains(line), "package.sh does not say: {line}");
    }
}

#[test]
fn the_scripts_ask_the_agent_for_readings_by_names_this_build_has() {
    let known = shipped_collectors();
    let mut asked = 0;

    for directory in ["env/docker", "env/scripts"] {
        for entry in fs::read_dir(workspace().join(directory)).expect("a directory") {
            let path = entry.expect("an entry").path();
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for (at, question) in text.match_indices(ASKED_FOR) {
                let name = text[at + question.len()..]
                    .split('"')
                    .next()
                    .unwrap_or_default();
                assert!(
                    known.iter().any(|known| known == name),
                    "{} asks for the reading of {name:?}, and no collector of this build is \
                     called that: the answer is an error the script reads as an empty reading",
                    path.display()
                );
                asked += 1;
            }
        }
    }

    assert!(asked >= 3, "only {asked} question(s) were checked");
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
