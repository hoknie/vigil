use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::container_finding::{ContainerFinding, build, finding_key, running};
use super::container_view::{ContainerView, Family};
use super::docker_socket_exposed::is_a_runtime_socket;
use crate::{Rule, RuleContext};

const OF_THIS_HOST: &[&str] = &[
    "/", "/boot", "/dev", "/etc", "/home", "/proc", "/root", "/srv", "/sys", "/usr", "/var",
];

const A_RUNTIME_PUTS_THERE: &[&str] = &[
    "/var/lib/docker/containers/",
    "/var/lib/containers/storage/",
    "/var/lib/kubelet/pods/",
    "/var/lib/docker/volumes/",
    "/var/lib/containerd/",
];

pub struct ContainerHostMount;

impl Rule for ContainerHostMount {
    fn name(&self) -> &'static str {
        "container_host_mount"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };

        let now = ContainerView::new(key, after);
        if !now.is(Family::Container) {
            return None;
        }

        let held = of_this_host(&now);
        if held.is_empty() {
            return None;
        }
        if let Some(before) = before
            && of_this_host(&ContainerView::new(key, before)) == held
        {
            return None;
        }

        Some(build(
            ContainerFinding {
                kind: KnownKind::ContainerHostMount,
                severity: severity(&held),
                rule: self.name(),
                key,
                finding_key: finding_key("host-mount", &now),
                object: "container",
                title: format!(
                    "The {} container running {} holds {} of this host: what it may write there, this host runs",
                    now.runtime(),
                    now.executable()
                        .unwrap_or("a program this agent may not read"),
                    held.join(", ")
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    running(&now),
                    Evidence {
                        kind: "note".into(),
                        value: format!("paths of this host inside it: {}", held.join(", ")),
                    },
                ],
            },
            ctx,
        ))
    }
}

fn of_this_host<'a>(view: &ContainerView<'a>) -> Vec<&'a str> {
    view.host_paths()
        .into_iter()
        .filter(|path| !is_a_runtime_socket(path))
        .filter(|path| !A_RUNTIME_PUTS_THERE.iter().any(|at| path.starts_with(at)))
        .filter(|path| under_a_watched_root(path))
        .collect()
}

fn under_a_watched_root(path: &str) -> bool {
    OF_THIS_HOST.iter().any(|root| match *root {
        "/" => path == "/",
        named => path == named || path.starts_with(&format!("{named}/")),
    })
}

fn severity(held: &[&str]) -> Severity {
    match held.contains(&"/") {
        true => Severity::Critical,
        false => Severity::High,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    const ORDINARY: &str = "00000000a80425fb";

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        ContainerHostMount.apply(change, &mut ctx)
    }

    fn holding(paths: &[&str]) -> Change {
        Change::Added {
            key: "container|3ab1c0f2d4e5".into(),
            after: fixture::container("/usr/sbin/nginx", ORDINARY, paths),
        }
    }

    #[test]
    fn a_container_holding_the_root_of_this_host_is_the_worst_of_them_and_is_named_so() {
        let finding = apply(&holding(&["/"])).expect("fires");

        assert_eq!(finding.finding_key, "container|host-mount|/usr/sbin/nginx");
        assert_eq!(finding.kind.as_str(), "container.host_mount");
        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn a_container_holding_the_configuration_of_this_host_is_reported_and_not_the_worst() {
        let finding = apply(&holding(&["/etc/nginx"])).expect("fires");

        assert_eq!(finding.severity, Severity::High);
        assert!(finding.title.contains("/etc/nginx"), "{}", finding.title);
    }

    #[test]
    fn the_files_a_runtime_binds_into_every_container_are_not_a_finding_about_any_of_them() {
        for plumbing in [
            "/var/lib/docker/containers/3ab1/resolv.conf",
            "/var/lib/docker/containers/3ab1/hostname",
            "/var/lib/kubelet/pods/9c1f/volumes/kube-api-access/token",
            "/var/lib/docker/volumes/site/_data",
        ] {
            assert!(
                apply(&holding(&[plumbing])).is_none(),
                "{plumbing} is inside every container a runtime starts, and a finding about it \
                 would be a finding about every container on every host"
            );
        }
    }

    #[test]
    fn a_directory_of_its_own_that_nothing_of_this_host_lives_in_is_not_a_finding() {
        assert!(apply(&holding(&["/opt/site/data"])).is_none());
        assert!(apply(&holding(&[])).is_none());
    }

    #[test]
    fn the_socket_of_the_runtime_is_left_to_the_rule_that_is_about_it() {
        assert!(
            apply(&holding(&["/var/run/docker.sock"])).is_none(),
            "a container holding the socket of its own runtime owns the host, and that is one \
             finding with a kind of its own rather than two about the same mount"
        );
    }

    #[test]
    fn a_container_whose_paths_did_not_move_is_not_reported_again() {
        let change = Change::Changed {
            key: "container|3ab1c0f2d4e5".into(),
            before: fixture::container("/usr/sbin/nginx", ORDINARY, &["/etc/nginx"]),
            after: fixture::container("/usr/sbin/nginx", "000001ffffffffff", &["/etc/nginx"]),
        };

        assert!(apply(&change).is_none());
    }
}
