use vigil_model::{KnownKind, Severity};

use super::subject::Subject;
use super::watching::Report;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifecycle {
    pub subject: Subject,
    pub rule: &'static str,
    pub thing: &'static str,
    pub object: &'static str,
    pub new: (KnownKind, Severity),
    pub removed: (KnownKind, Severity),
    pub changed: (KnownKind, Severity),
    pub matters: &'static [&'static str],
}

impl Lifecycle {
    pub const ALL: [Lifecycle; 6] = [IMAGES, VOLUMES, NETWORKS, PROJECTS, PODS, SECRETS];

    pub fn reported(&self, report: &Report) -> bool {
        match self.subject {
            Subject::Image => report.images,
            Subject::Volume => report.volumes,
            Subject::Network => report.networks,
            Subject::Project => report.projects,
            Subject::Pod => report.pods,
            Subject::Secret => report.secrets,
            _ => false,
        }
    }
}

const IMAGES: Lifecycle = Lifecycle {
    subject: Subject::Image,
    rule: "engine_images",
    thing: "image",
    object: "image",
    new: (KnownKind::ContainerImageNew, Severity::Low),
    removed: (KnownKind::ContainerImageRemoved, Severity::Low),
    changed: (KnownKind::ContainerImageChanged, Severity::Medium),
    matters: &[],
};

const VOLUMES: Lifecycle = Lifecycle {
    subject: Subject::Volume,
    rule: "engine_volumes",
    thing: "volume",
    object: "volume",
    new: (KnownKind::ContainerVolumeNew, Severity::Low),
    removed: (KnownKind::ContainerVolumeRemoved, Severity::Low),
    changed: (KnownKind::ContainerVolumeChanged, Severity::Low),
    matters: &["driver", "mountpoint", "device"],
};

const NETWORKS: Lifecycle = Lifecycle {
    subject: Subject::Network,
    rule: "engine_networks",
    thing: "network",
    object: "network",
    new: (KnownKind::ContainerNetworkNew, Severity::Low),
    removed: (KnownKind::ContainerNetworkRemoved, Severity::Low),
    changed: (KnownKind::ContainerNetworkChanged, Severity::Medium),
    matters: &["driver", "subnets", "internal"],
};

const PROJECTS: Lifecycle = Lifecycle {
    subject: Subject::Project,
    rule: "engine_projects",
    thing: "compose project",
    object: "compose_project",
    new: (KnownKind::ContainerProjectNew, Severity::Low),
    removed: (KnownKind::ContainerProjectRemoved, Severity::Low),
    changed: (KnownKind::ContainerProjectChanged, Severity::Low),
    matters: &["services"],
};

const PODS: Lifecycle = Lifecycle {
    subject: Subject::Pod,
    rule: "engine_pods",
    thing: "pod",
    object: "pod",
    new: (KnownKind::ContainerPodNew, Severity::Low),
    removed: (KnownKind::ContainerPodRemoved, Severity::Low),
    changed: (KnownKind::ContainerPodChanged, Severity::Low),
    matters: &["containers", "networks"],
};

const SECRETS: Lifecycle = Lifecycle {
    subject: Subject::Secret,
    rule: "engine_secrets",
    thing: "secret",
    object: "secret",
    new: (KnownKind::ContainerSecretNew, Severity::Medium),
    removed: (KnownKind::ContainerSecretRemoved, Severity::Medium),
    changed: (KnownKind::ContainerSecretChanged, Severity::Medium),
    matters: &["id", "driver", "updated_at"],
};

#[cfg(test)]
mod tests {
    use super::*;

    fn of(subject: Subject) -> Option<Lifecycle> {
        Lifecycle::ALL
            .into_iter()
            .find(|lifecycle| lifecycle.subject == subject)
    }

    #[test]
    fn every_subject_that_comes_and_goes_has_three_kinds_of_its_own_and_one_toggle() {
        let mut rules: Vec<&str> = Lifecycle::ALL.iter().map(|one| one.rule).collect();
        rules.sort_unstable();
        rules.dedup();
        assert_eq!(rules.len(), Lifecycle::ALL.len());

        let silenced = Report {
            images: false,
            volumes: false,
            networks: false,
            projects: false,
            pods: false,
            secrets: false,
        };
        for lifecycle in Lifecycle::ALL {
            let thing = lifecycle
                .new
                .0
                .as_str()
                .split('.')
                .nth(1)
                .unwrap_or_default();
            for (kind, _) in [&lifecycle.new, &lifecycle.removed, &lifecycle.changed] {
                assert!(
                    kind.as_str().starts_with(&format!("container.{thing}.")),
                    "{}: the three kinds of one subject share its name",
                    kind.as_str()
                );
            }
            assert!(lifecycle.reported(&Report::default()));
            assert!(
                !lifecycle.reported(&silenced),
                "{}: a host whose images change on every deploy turns this off by name",
                lifecycle.rule
            );
        }
    }

    #[test]
    fn what_goes_under_a_name_it_kept_is_named_field_by_field_and_nothing_else_counts() {
        assert_eq!(
            of(Subject::Network).map(|one| one.matters),
            Some(&["driver", "subnets", "internal"][..])
        );
        assert_eq!(
            of(Subject::Image).map(|one| one.matters),
            Some(&[][..]),
            "an image is its id: nothing under one id changes, and a tag that moved is two \
             images, which the rule about moved tags reads together"
        );
        assert_eq!(of(Subject::Container), None);
        assert_eq!(of(Subject::Registry), None);
    }
}
