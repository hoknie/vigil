use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::container_finding::{ContainerFinding, build, finding_key, running};
use crate::types::{ContainerView, Family};
use vigil_rules::{Rule, RuleContext};

const RUNTIME_SOCKETS: &[&str] = &[
    "/var/run/docker.sock",
    "/run/docker.sock",
    "/run/podman/podman.sock",
    "/var/run/podman/podman.sock",
    "/run/containerd/containerd.sock",
    "/var/run/crio/crio.sock",
];

pub fn is_a_runtime_socket(path: &str) -> bool {
    RUNTIME_SOCKETS.contains(&path)
}

pub struct ContainerDockerSocketExposed;

impl Rule for ContainerDockerSocketExposed {
    fn name(&self) -> &'static str {
        "container_docker_socket_exposed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };

        let now = ContainerView::new(key, after);
        let was = before.map(|before| ContainerView::new(key, before));

        match now.family()? {
            Family::Socket => self.on_this_host(&now, was.as_ref(), key, before, after, ctx),
            Family::Container => self.inside(&now, was.as_ref(), key, before, after, ctx),
        }
    }
}

impl ContainerDockerSocketExposed {
    fn on_this_host(
        &self,
        now: &ContainerView<'_>,
        was: Option<&ContainerView<'_>>,
        key: &str,
        before: Option<&serde_json::Value>,
        after: &serde_json::Value,
        ctx: &mut RuleContext<'_>,
    ) -> Option<Finding> {
        if !now.reachable_by_anyone() {
            return None;
        }
        if was.is_some_and(ContainerView::reachable_by_anyone) {
            return None;
        }

        Some(build(
            ContainerFinding {
                kind: KnownKind::ContainerDockerSocketExposed,
                severity: Severity::Critical,
                rule: self.name(),
                key,
                finding_key: format!("container|docker-socket|{}", now.path()),
                object: "socket",
                title: format!(
                    "{} is mode {} on this host: anyone who can open it starts a container as root, and a container started as root is this host",
                    now.path(),
                    now.mode()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![Evidence {
                    kind: "note".into(),
                    value: format!(
                        "a runtime ships this socket as 0660 owned by root and a group; mode {} gives it to every account on this host",
                        now.mode()
                    ),
                }],
            },
            ctx,
        ))
    }

    fn inside(
        &self,
        now: &ContainerView<'_>,
        was: Option<&ContainerView<'_>>,
        key: &str,
        before: Option<&serde_json::Value>,
        after: &serde_json::Value,
        ctx: &mut RuleContext<'_>,
    ) -> Option<Finding> {
        let held = sockets_in(now);
        if held.is_empty() {
            return None;
        }
        if was.is_some_and(|was| sockets_in(was) == held) {
            return None;
        }

        Some(build(
            ContainerFinding {
                kind: KnownKind::ContainerDockerSocketExposed,
                severity: Severity::Critical,
                rule: self.name(),
                key,
                finding_key: finding_key("docker-socket", now),
                object: "container",
                title: format!(
                    "The {} container running {} holds {} of this host: from inside it, a container can be started as root on this host",
                    now.runtime(),
                    now.executable().unwrap_or("a program this agent may not read"),
                    held.join(", ")
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    running(now),
                    Evidence {
                        kind: "note".into(),
                        value: "the socket of a container runtime is not an api with permissions on it: whatever can write to it can mount the root of this host into a container of its own".into(),
                    },
                ],
            },
            ctx,
        ))
    }
}

fn sockets_in<'a>(view: &ContainerView<'a>) -> Vec<&'a str> {
    view.host_paths()
        .into_iter()
        .filter(|path| is_a_runtime_socket(path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    const ORDINARY: &str = "00000000a80425fb";

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        ContainerDockerSocketExposed.apply(change, &mut ctx)
    }

    fn socket_is(mode: &str) -> Change {
        Change::Added {
            key: "container-socket|/run/docker.sock".into(),
            after: fixture::runtime_socket("/run/docker.sock", mode),
        }
    }

    fn container_holds(paths: &[&str]) -> Change {
        Change::Added {
            key: "container|3ab1c0f2d4e5".into(),
            after: fixture::container("/usr/local/bin/agent", ORDINARY, paths),
        }
    }

    #[test]
    fn a_socket_every_account_on_this_host_may_write_to_is_the_host_handed_over() {
        let finding = apply(&socket_is("0666")).expect("fires");

        assert_eq!(
            finding.finding_key,
            "container|docker-socket|/run/docker.sock"
        );
        assert_eq!(finding.kind.as_str(), "container.docker_socket_exposed");
        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn the_mode_a_runtime_ships_is_not_a_finding_on_every_host_that_has_one() {
        assert!(apply(&socket_is("0660")).is_none());
        assert!(apply(&socket_is("0600")).is_none());
    }

    #[test]
    fn a_container_holding_the_socket_of_its_runtime_is_the_same_kind_from_the_other_side() {
        let finding = apply(&container_holds(&["/var/run/docker.sock"])).expect("fires");

        assert_eq!(
            finding.finding_key,
            "container|docker-socket|/usr/local/bin/agent"
        );
        assert_eq!(finding.kind.as_str(), "container.docker_socket_exposed");
        assert_eq!(
            finding.subject.object, "container",
            "one kind, two objects: the socket of this host and the container holding it are \
             two things to suppress and two things to fix"
        );
    }

    #[test]
    fn a_container_holding_nothing_of_the_runtime_says_nothing_here() {
        assert!(apply(&container_holds(&["/etc/nginx", "/srv/www"])).is_none());
        assert!(apply(&container_holds(&[])).is_none());
    }

    #[test]
    fn a_socket_that_was_already_open_to_everyone_is_not_reported_again() {
        let change = Change::Changed {
            key: "container-socket|/run/docker.sock".into(),
            before: fixture::runtime_socket("/run/docker.sock", "0666"),
            after: fixture::runtime_socket("/run/docker.sock", "0777"),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_socket_that_was_put_back_the_way_it_shipped_says_nothing() {
        let change = Change::Changed {
            key: "container-socket|/run/docker.sock".into(),
            before: fixture::runtime_socket("/run/docker.sock", "0666"),
            after: fixture::runtime_socket("/run/docker.sock", "0660"),
        };

        assert!(
            apply(&change).is_none(),
            "this vocabulary has no kind for a socket that was closed again, and inventing one \
             in a rule would be a change to a published contract"
        );
    }
}
