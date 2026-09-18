use vigil_model::Snapshot;

use super::{docker, podman};
use crate::parsers::{
    EngineReading, EnginesReading, engines_snapshot, parse_daemon_json, parse_registries_conf,
};
use crate::types::{Dump, Engine, FAILED, Subject};

pub fn engines() -> Snapshot {
    let docker = docker::dump();
    let podman = podman::dump();

    read(&docker, &podman)
}

pub fn read(docker: &Dump, podman: &Dump) -> Snapshot {
    let docker = EngineReading {
        engine: Engine::Docker,
        dump: Some(docker),
        registries: parse_daemon_json(docker::DAEMON_JSON, Engine::Docker.registries_file())
            .expect("the sample daemon.json parses"),
        registries_read: true,
    };
    let podman = EngineReading {
        engine: Engine::Podman,
        dump: Some(podman),
        registries: parse_registries_conf(
            podman::REGISTRIES_CONF,
            Engine::Podman.registries_file(),
        ),
        registries_read: true,
    };

    engines_snapshot(
        docker::AT,
        &EnginesReading {
            engines: &[docker, podman],
        },
    )
}

pub fn only_docker() -> Snapshot {
    let docker = docker::dump();
    let podman = Dump::absent(
        Engine::Podman.name(),
        docker::AT,
        docker::DEADLINE_SECONDS,
        format!(
            "podman is not on this host (looked in {})",
            Engine::Podman.places().join(", ")
        ),
    );

    read(&docker, &podman)
}

pub fn docker_silent_on(subject: Subject) -> Snapshot {
    let mut docker = docker::dump();
    if let Some(answer) = docker.asked.get_mut(subject.as_str()) {
        answer.state = FAILED.to_string();
        answer.printed = String::new();
        answer.why =
            Some("Cannot connect to the Docker daemon at unix:///var/run/docker.sock".to_string());
    }

    read(&docker, &podman::dump())
}
