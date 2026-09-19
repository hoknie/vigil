use std::fs;
use std::path::{Path, PathBuf};

use vigil_config::Installation;
use vigil_module::Module;

const JOBS: &str = "packaging/macos/launchd";

const POSTINSTALL: &str = "packaging/macos/scripts/postinstall";

const PACKAGE: &str = "env/scripts/package-macos.sh";

type Job = (String, String);

fn value(job: &Job, key: &str) -> Option<String> {
    let after = job.1.split(&format!("<key>{key}</key>")).nth(1)?;
    let opened = after.trim_start();
    let tag = opened.strip_prefix('<')?.split('>').next()?.to_string();
    let inner = opened.strip_prefix(&format!("<{tag}>"))?;
    Some(inner.split(&format!("</{tag}>")).next()?.trim().to_string())
}

fn arguments(job: &Job) -> Vec<String> {
    let Some(array) = value(job, "ProgramArguments") else {
        return Vec::new();
    };
    array
        .split("<string>")
        .skip(1)
        .filter_map(|piece| piece.split("</string>").next())
        .map(str::to_string)
        .collect()
}

fn jobs() -> Vec<Job> {
    let directory = workspace().join(JOBS);
    let mut read: Vec<Job> = fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "plist"))
        .map(|path| {
            (
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string(),
                fs::read_to_string(&path).expect("a plist"),
            )
        })
        .collect();
    read.sort();
    assert_eq!(read.len(), 4, "vigild, firewall, containers and launches");
    read
}

fn job(file: &str) -> Job {
    jobs()
        .into_iter()
        .find(|job| job.0 == file)
        .unwrap_or_else(|| panic!("{file} is not in {JOBS}"))
}

#[test]
fn every_job_the_package_installs_is_labelled_by_the_name_of_its_file() {
    for job in jobs() {
        assert_eq!(
            value(&job, "Label").map(|label| format!("{label}.plist")),
            Some(job.0.clone()),
            "launchctl bootstrap takes the file and bootout takes the label; the two are one \
             name or the command that stops a reading stops nothing"
        );
        assert_eq!(
            value(&job, "UserName").as_deref(),
            Some("root"),
            "{}",
            job.0
        );
    }
}

#[test]
fn every_program_a_job_runs_is_named_by_its_absolute_path() {
    for job in jobs() {
        let arguments = arguments(&job);
        let program = arguments.first().expect("a program");

        assert!(
            Path::new(program).is_absolute(),
            "{} runs {program:?}: launchd looks a bare name up in a PATH nobody chose",
            job.0
        );
        assert!(
            program.starts_with("/usr/local/"),
            "{}: what this package installs lives under /usr/local, because the system volume \
             of a Mac is read-only: {program}",
            job.0
        );
    }
}

#[test]
fn the_daemon_is_started_with_the_configuration_a_mac_installs_and_kept_running() {
    let daemon = job("vigil.vigild.plist");

    assert_eq!(
        arguments(&daemon),
        vec![
            "/usr/local/sbin/vigild".to_string(),
            Installation::MACOS.configuration.to_string()
        ]
    );
    assert!(daemon.1.contains("<key>KeepAlive</key>\n\t<true/>"));
    assert!(daemon.1.contains("<key>RunAtLoad</key>\n\t<true/>"));
}

#[test]
fn what_the_jobs_say_about_themselves_is_written_where_a_mac_keeps_the_logs_of_this_agent() {
    for job in jobs() {
        let written = value(&job, "StandardErrorPath").expect("a log");

        assert!(
            written.starts_with(&format!("{}/", Installation::MACOS.log_directory)),
            "{}: a daemon of launchd with no StandardErrorPath says everything to nobody: \
             {written}",
            job.0
        );
    }
}

#[test]
fn the_firewall_job_writes_the_reading_at_the_period_the_collector_reads_it() {
    let declared = vigil_firewall::Firewall.every_seconds();

    assert_eq!(
        value(&job("vigil.firewall.plist"), "StartInterval"),
        Some(declared.to_string()),
        "the job and the collector are two halves of one reading, as the timer is on Linux"
    );
}

#[test]
fn the_engines_job_asks_the_engines_at_the_period_the_collector_reads_them() {
    let declared = vigil_engines::Engines.every_seconds();
    let containers = job("vigil.containers.plist");

    assert_eq!(
        value(&containers, "StartInterval"),
        Some(declared.to_string())
    );
    assert_eq!(
        arguments(&containers),
        vec![
            "/usr/local/libexec/vigil/vigil-container-dump".to_string(),
            "--directory".to_string(),
            format!("{}/containers", Installation::MACOS.state_directory),
        ],
        "this package's own binary and one fixed path, as on Linux"
    );
}

#[test]
fn every_directory_a_job_writes_into_is_made_by_the_installer() {
    let postinstall = fs::read_to_string(workspace().join(POSTINSTALL)).expect("postinstall");

    for job in jobs() {
        let arguments = arguments(&job);
        let Some(at) = arguments.iter().position(|word| word == "--directory") else {
            continue;
        };
        let directory = &arguments[at + 1];
        let under = directory
            .strip_prefix(&format!("{}/", Installation::MACOS.state_directory))
            .unwrap_or_else(|| panic!("{}: {directory} is not under the state", job.0));

        assert!(
            postinstall.contains(&format!("\"$STATE/{under}\"")),
            "{}: {directory} is not made by the installer, and the first run fails with \
             nothing to say",
            job.0
        );
    }
    assert!(postinstall.contains(&format!(
        "STATE=\"$ROOT{}\"",
        Installation::MACOS.state_directory
    )));
    assert!(postinstall.contains(&format!(
        "ETC=\"$ROOT{}\"",
        Installation::MACOS.configuration_directory
    )));
}

#[test]
fn the_package_moves_the_shipped_configuration_to_the_places_this_build_names_for_a_mac() {
    let script = fs::read_to_string(workspace().join(PACKAGE)).expect("package-macos.sh");
    let socket_directory = Installation::MACOS
        .socket
        .rsplit_once('/')
        .map(|(directory, _)| directory)
        .expect("a directory");

    for (name, place) in [
        ("MACOS_ETC", Installation::MACOS.configuration_directory),
        ("MACOS_STATE", Installation::MACOS.state_directory),
        ("MACOS_LOGS", Installation::MACOS.log_directory),
        ("MACOS_RUN", socket_directory),
        ("LINUX_ETC", Installation::LINUX.configuration_directory),
        ("LINUX_STATE", Installation::LINUX.state_directory),
        ("LINUX_LOGS", Installation::LINUX.log_directory),
    ] {
        assert!(
            script.contains(&format!("{name}=\"{place}\"")),
            "the package and `vigild configure` write one configuration for a Mac; {name} in \
             {PACKAGE} is not {place}"
        );
    }
}

#[test]
fn the_installer_never_writes_over_a_configuration_file_it_finds() {
    let postinstall = fs::read_to_string(workspace().join(POSTINSTALL)).expect("postinstall");

    assert!(
        postinstall.contains("\"$installed.new\""),
        "a file an operator edited is kept, and the new defaults go beside it"
    );
    assert!(
        !postinstall.contains("cp \"$shipped\" \"$installed\"")
            && postinstall.contains("[ -f \"$before\" ] && cmp -s \"$before\" \"$installed\""),
        "a file is brought to the new defaults only when it is byte for byte what the last \
         version shipped"
    );
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root")
}
