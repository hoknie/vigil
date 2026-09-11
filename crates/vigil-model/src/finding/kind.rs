use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum Kind {
    Known(KnownKind),
    Unknown(String),
}

impl Kind {
    pub fn as_str(&self) -> &str {
        match self {
            Kind::Known(k) => k.as_str(),
            Kind::Unknown(s) => s,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for Kind {
    fn from(s: String) -> Self {
        match KnownKind::parse(&s) {
            Some(k) => Kind::Known(k),
            None => Kind::Unknown(s),
        }
    }
}

impl From<Kind> for String {
    fn from(k: Kind) -> Self {
        k.as_str().to_string()
    }
}

macro_rules! kinds {
    ($($variant:ident => $wire:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum KnownKind { $($variant),* }

        impl KnownKind {
            pub const ALL: &'static [KnownKind] = &[$(KnownKind::$variant),*];

            pub fn as_str(self) -> &'static str {
                match self { $(KnownKind::$variant => $wire),* }
            }

            pub fn parse(s: &str) -> Option<Self> {
                match s { $($wire => Some(KnownKind::$variant),)* _ => None }
            }
        }
    };
}

kinds! {
    PortListenNew => "port.listen.new",
    PortListenRemoved => "port.listen.removed",
    PortListenExposed => "port.listen.exposed",
    PortListenOwnerChanged => "port.listen.owner_changed",
    UserAccountNew => "user.account.new",
    UserAccountRemoved => "user.account.removed",
    UserAccountUid0 => "user.account.uid0",
    UserAccountUnlocked => "user.account.unlocked",
    UserGroupPrivilegedMemberAdded => "user.group.privileged_member_added",
    UserSshkeyAdded => "user.sshkey.added",
    UserSshkeyRemoved => "user.sshkey.removed",
    UserPasswordChanged => "user.password.changed",
    AuthFailureBurst => "auth.failure.burst",
    AuthSuccessAfterBurst => "auth.success_after_burst",
    AuthNewSource => "auth.new_source",
    AuthSudoUsed => "auth.sudo.used",
    AuthSuUsed => "auth.su.used",
    ProcessUnexpectedParent => "process.unexpected_parent",
    ProcessFromWritablePath => "process.from_writable_path",
    ProcessBinaryDeleted => "process.binary_deleted",
    ProcessRootNew => "process.root.new",
    ExecFirstSeenForUser => "exec.first_seen_for_user",
    ExecFromWritablePath => "exec.from_writable_path",
    FileChanged => "file.changed",
    FilePermissionsChanged => "file.permissions_changed",
    FileSuidNew => "file.suid.new",
    FilePathWritableByAll => "file.path_writable_by_all",
    FileSshdConfigWeakened => "file.sshd_config.weakened",
    FileMacModeChanged => "file.mac_mode_changed",
    PersistenceUnitNew => "persistence.unit.new",
    PersistenceTimerNew => "persistence.timer.new",
    PersistenceCronNew => "persistence.cron.new",
    PersistencePreloadChanged => "persistence.preload_changed",
    PersistenceKernelModuleLoaded => "persistence.kernel_module.loaded",
    PersistenceShellProfileChanged => "persistence.shell_profile_changed",
    ContainerPrivileged => "container.privileged",
    ContainerHostMount => "container.host_mount",
    ContainerDockerSocketExposed => "container.docker_socket_exposed",
    ContainerImageUnknown => "container.image_unknown",
    ResourceDiskLow => "resource.disk_low",
    ResourceInodeLow => "resource.inode_low",
    ResourceReboot => "resource.reboot",
    ResourceClockSkew => "resource.clock_skew",
    AgentBaselineReady => "agent.baseline.ready",
    AgentCollectorDegraded => "agent.collector.degraded",
    AgentBufferDropping => "agent.buffer.dropping",
    AgentBudgetExceeded => "agent.budget.exceeded",
    AgentBudgetRecovered => "agent.budget.recovered",
    AgentCloneSuspected => "agent.clone_suspected",
    AgentStoreDamaged => "agent.store.damaged",
    AgentReceiverRefused => "agent.receiver_refused",
}

impl KnownKind {
    pub fn resolves(self) -> Option<KnownKind> {
        match self {
            KnownKind::PortListenRemoved => Some(KnownKind::PortListenNew),
            KnownKind::UserAccountRemoved => Some(KnownKind::UserAccountNew),
            KnownKind::UserSshkeyRemoved => Some(KnownKind::UserSshkeyAdded),
            KnownKind::AgentBudgetRecovered => Some(KnownKind::AgentBudgetExceeded),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closing_kind_names_the_kind_it_closes_and_the_pair_is_not_circular() {
        assert_eq!(
            KnownKind::PortListenRemoved.resolves(),
            Some(KnownKind::PortListenNew)
        );
        assert_eq!(KnownKind::PortListenNew.resolves(), None);

        for kind in KnownKind::ALL {
            if let Some(closed) = kind.resolves() {
                assert_eq!(
                    closed.resolves(),
                    None,
                    "{} closes a closing kind",
                    kind.as_str()
                );
                assert_ne!(closed, *kind, "{} closes itself", kind.as_str());
            }
        }
    }

    #[test]
    fn the_cost_of_the_agent_going_over_its_ceiling_is_closed_by_its_own_kind() {
        assert_eq!(
            KnownKind::AgentBudgetRecovered.resolves(),
            Some(KnownKind::AgentBudgetExceeded),
            "coming back under the ceiling is an event of its own, not the absence of one"
        );
        assert_eq!(KnownKind::AgentBudgetExceeded.resolves(), None);
    }

    #[test]
    fn every_kind_round_trips_through_its_wire_form() {
        for kind in KnownKind::ALL {
            let wire = kind.as_str();
            assert_eq!(
                KnownKind::parse(wire),
                Some(*kind),
                "{wire} did not round-trip"
            );
        }
    }

    #[test]
    fn wire_names_are_unique_and_namespaced() {
        let mut seen: Vec<&str> = KnownKind::ALL.iter().map(|k| k.as_str()).collect();
        let total = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), total, "two kinds share a wire name");
        for kind in KnownKind::ALL {
            assert!(
                kind.as_str().contains('.'),
                "{} is not <group>.<…>",
                kind.as_str()
            );
        }
    }

    #[test]
    fn an_unknown_kind_survives_the_trip_instead_of_being_dropped() {
        let from_the_future = "port.listen.moved_to_wireguard";
        let parsed: Kind = from_the_future.to_string().into();
        assert_eq!(parsed, Kind::Unknown(from_the_future.to_string()));
        assert_eq!(String::from(parsed), from_the_future);
    }

    #[test]
    fn the_agent_reports_on_itself_in_the_same_vocabulary() {
        assert!(
            KnownKind::ALL
                .iter()
                .any(|k| k.as_str().starts_with("agent.")),
            "a degraded collector must be expressible as a finding"
        );
    }
}
