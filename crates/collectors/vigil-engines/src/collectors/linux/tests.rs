use std::fs;
use std::path::{Path, PathBuf};

use vigil_collect::{Collector, Health};

use super::collector::EnginesCollector;
use crate::fixture::{docker, podman};
use crate::types::{Dump, Engine, Watching};

const AT: &str = "2026-09-17T09:00:02.000Z";

fn workspace(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("vigil-engines-{}-{named}", std::process::id()));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).expect("a directory of its own");
    at
}

fn written(at: &Path, dump: &Dump) {
    let engine = Engine::named(&dump.engine).expect("a known engine");
    fs::write(
        at.join(engine.dump_file()),
        serde_json::to_vec_pretty(dump).expect("plain data"),
    )
    .expect("write");
}

fn reading(at: &Path) -> EnginesCollector {
    EnginesCollector::with_paths(|| AT.to_string(), at, Watching::default())
        .reading_registries_from(&[])
}

#[test]
fn a_dump_of_both_engines_reads_as_one_snapshot_named_after_the_reading_it_is() {
    let at = workspace("both");
    written(&at, &docker::dump());
    written(&at, &podman::dump());

    let collector = reading(&at);
    let read = collector.collect().expect("reads");

    assert_eq!(collector.name(), "containers-engines");
    assert_eq!(read.source, "containers-engines");
    assert_eq!(read.taken_at, AT);
    assert_eq!(collector.available(), Health::Ok);
    assert!(read.items.contains_key("docker|image|sha256:18ad9bdc4c87"));
    assert!(read.items.contains_key("podman|network|podman1"));
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_host_where_the_timer_has_never_run_says_the_reading_is_unknown_rather_than_empty() {
    let at = workspace("nothing");

    let collector = reading(&at);

    assert!(
        collector.collect().is_err(),
        "a reading with no dump behind it would publish a host with no images, no volumes and no networks, and the differ would report every one of them as removed"
    );
    match collector.available() {
        Health::Unavailable(why) => {
            assert!(why.contains("vigil-containers.timer"), "{why}");
            assert!(
                why.contains("not the same as a host with no containers"),
                "{why}"
            );
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

fn not_installed(engine: Engine) -> Dump {
    Dump::absent(
        engine.name(),
        AT,
        10,
        format!(
            "{} is not on this host (looked in {})",
            engine.name(),
            engine.places().join(", ")
        ),
    )
}

#[test]
fn an_engine_that_is_not_installed_leaves_the_reading_healthy() {
    let at = workspace("absent");
    written(&at, &docker::dump());
    written(&at, &not_installed(Engine::Podman));

    let collector = reading(&at);
    let read = collector.collect().expect("reads");

    assert_eq!(read.items["podman|engine|podman"]["present"], false);
    assert_eq!(
        collector.available(),
        Health::Ok,
        "an engine this host does not have is an answer, recorded as a row with present: \
         false, and not a failure of the reading"
    );
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn docker_that_is_not_installed_leaves_the_reading_healthy_as_podman_does() {
    let at = workspace("absent-docker");
    written(&at, &not_installed(Engine::Docker));
    written(&at, &podman::dump());

    let collector = reading(&at);
    let read = collector.collect().expect("reads");

    assert_eq!(read.items["docker|engine|docker"]["present"], false);
    assert_eq!(collector.available(), Health::Ok);
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_host_where_none_of_the_listed_engines_is_installed_is_healthy_and_holds_no_engine() {
    let at = workspace("none-installed");
    written(&at, &not_installed(Engine::Docker));
    written(&at, &not_installed(Engine::Podman));

    let collector = reading(&at);
    let read = collector.collect().expect("reads");

    assert_eq!(read.items["docker|engine|docker"]["present"], false);
    assert_eq!(read.items["podman|engine|podman"]["present"], false);
    assert_eq!(collector.available(), Health::Ok);
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn an_installed_engine_that_did_not_reply_still_degrades_it_beside_one_that_is_not_installed() {
    let at = workspace("silent-beside-absent");
    let mut broken = docker::dump();
    let answer = broken.asked.get_mut("container").expect("asked for");
    answer.state = crate::types::TIMED_OUT.to_string();
    answer.printed = String::new();
    written(&at, &broken);
    written(&at, &not_installed(Engine::Podman));

    let collector = reading(&at);

    match collector.available() {
        Health::Degraded(why) => {
            assert!(why.contains("docker did not answer for container"), "{why}");
            assert!(
                !why.contains("podman"),
                "the engine that is not installed has nothing to complain about: {why}"
            );
        }
        other => panic!("an installed engine that did not reply is a failure: {other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn an_engine_that_is_not_installed_but_whose_dump_is_stale_still_degrades_it() {
    let at = workspace("absent-stale");
    written(&at, &docker::dump());
    written(&at, &not_installed(Engine::Podman));
    let long_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(4_000);
    fs::File::options()
        .write(true)
        .open(at.join(Engine::Podman.dump_file()))
        .expect("the dump")
        .set_times(fs::FileTimes::new().set_modified(long_ago))
        .expect("an older mtime");

    let collector = reading(&at);

    match collector.available() {
        Health::Degraded(why) => assert!(why.contains("podman.json was written"), "{why}"),
        other => panic!(
            "an old answer of absent may no longer be true, and the timer that should have \
             renewed it is not firing: {other:?}"
        ),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn an_engine_that_refused_one_of_its_commands_is_named_with_what_it_said() {
    let at = workspace("refused");
    let mut broken = docker::dump();
    let answer = broken.asked.get_mut("volume").expect("asked for");
    answer.state = crate::types::FAILED.to_string();
    answer.printed = String::new();
    answer.why = Some("Cannot connect to the Docker daemon at unix:///var/run/docker.sock".into());
    written(&at, &broken);
    written(&at, &podman::dump());

    let collector = reading(&at);
    let read = collector.collect().expect("reads");

    assert!(
        !read
            .items
            .keys()
            .any(|key| key.starts_with("docker|volume|")),
        "a command that failed must leave no row, or the next reading reports every volume \
         of this host as new"
    );
    match collector.available() {
        Health::Degraded(why) => {
            assert!(why.contains("did not answer for volume"), "{why}");
            assert!(why.contains("Cannot connect"), "{why}");
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_dump_that_is_not_the_document_this_collector_reads_is_a_refusal_naming_the_file() {
    let at = workspace("rubbish");
    fs::write(at.join("docker.json"), b"docker: command not found\n").expect("write");
    fs::write(at.join("podman.json"), b"{}").expect("write");

    let collector = reading(&at);

    assert!(collector.collect().is_err());
    match collector.available() {
        Health::Degraded(why) | Health::Unavailable(why) => {
            assert!(why.contains("docker.json"), "{why}")
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_dump_older_than_twice_its_period_is_still_read_and_the_health_line_says_how_old_it_is() {
    let at = workspace("stale");
    written(&at, &docker::dump());
    written(&at, &podman::dump());
    let long_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(4_000);
    for engine in Engine::ALL {
        let file = fs::File::options()
            .write(true)
            .open(at.join(engine.dump_file()))
            .expect("the dump");
        file.set_times(fs::FileTimes::new().set_modified(long_ago))
            .expect("an older mtime");
    }

    let collector = reading(&at);

    assert!(
        collector.collect().is_ok(),
        "an old reading is still the last thing known about this host; the health line says \
         it is old and the reader decides what to do with it"
    );
    match collector.available() {
        Health::Degraded(why) => {
            assert!(why.contains("was written"), "{why}");
            assert!(why.contains("240"), "{why}");
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn the_registries_of_this_host_are_read_by_the_agent_out_of_the_files_it_is_pointed_at() {
    let at = workspace("registries");
    written(&at, &docker::dump());
    written(&at, &podman::dump());
    let daemon = at.join("daemon.json");
    let conf = at.join("registries.conf");
    fs::write(&daemon, docker::DAEMON_JSON).expect("write");
    fs::write(&conf, podman::REGISTRIES_CONF).expect("write");

    let read = EnginesCollector::with_paths(|| AT.to_string(), &at, Watching::default())
        .reading_registries_from(&[
            (Engine::Docker, daemon.to_str().expect("utf-8")),
            (Engine::Podman, conf.to_str().expect("utf-8")),
        ])
        .collect()
        .expect("reads");

    assert_eq!(
        read.items["docker|registry|registry.local:5000"]["insecure"],
        true
    );
    assert_eq!(read.items["podman|registry|docker.io"]["role"], "search");
    let _ = fs::remove_dir_all(&at);
}
