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
    PersistenceCronRemoved => "persistence.cron.removed",
    PersistenceCronChanged => "persistence.cron.changed",
    PersistencePreloadChanged => "persistence.preload_changed",
    PersistenceKernelModuleLoaded => "persistence.kernel_module.loaded",
    PersistenceShellProfileChanged => "persistence.shell_profile_changed",
    FirewallDisabled => "firewall.disabled",
    FirewallEnabled => "firewall.enabled",
    FirewallRulesetFlushed => "firewall.ruleset_flushed",
    FirewallPolicyWeakened => "firewall.policy_weakened",
    ContainerPrivileged => "container.privileged",
    ContainerHostMount => "container.host_mount",
    ContainerDockerSocketExposed => "container.docker_socket_exposed",
    ContainerImageUnknown => "container.image_unknown",
    ContainerImageNew => "container.image.new",
    ContainerImageRemoved => "container.image.removed",
    ContainerImageChanged => "container.image.changed",
    ContainerImageUntaggedInUse => "container.image.untagged_in_use",
    ContainerVolumeNew => "container.volume.new",
    ContainerVolumeRemoved => "container.volume.removed",
    ContainerVolumeChanged => "container.volume.changed",
    ContainerVolumeHostMount => "container.volume.host_mount",
    ContainerNetworkNew => "container.network.new",
    ContainerNetworkRemoved => "container.network.removed",
    ContainerNetworkChanged => "container.network.changed",
    ContainerNetworkHostMode => "container.network.host_mode",
    ContainerProjectNew => "container.project.new",
    ContainerProjectRemoved => "container.project.removed",
    ContainerProjectChanged => "container.project.changed",
    ContainerProjectPrivilegedService => "container.project.privileged_service",
    ContainerPodNew => "container.pod.new",
    ContainerPodRemoved => "container.pod.removed",
    ContainerPodChanged => "container.pod.changed",
    ContainerSecretNew => "container.secret.new",
    ContainerSecretRemoved => "container.secret.removed",
    ContainerSecretChanged => "container.secret.changed",
    ContainerRegistryInsecure => "container.registry.insecure",
    ResourceDiskLow => "resource.disk_low",
    ResourceInodeLow => "resource.inode_low",
    ResourceReboot => "resource.reboot",
    ResourceClockSkew => "resource.clock_skew",
    AgentBaselineReady => "agent.baseline.ready",
    AgentCollectorDegraded => "agent.collector.degraded",
    AgentCollectorRecovered => "agent.collector.recovered",
    AgentBufferDropping => "agent.buffer.dropping",
    AgentBufferDrained => "agent.buffer.drained",
    AgentBudgetExceeded => "agent.budget.exceeded",
    AgentBudgetRecovered => "agent.budget.recovered",
    AgentCloneSuspected => "agent.clone_suspected",
    AgentStoreDamaged => "agent.store.damaged",
    AgentReceiverRefused => "agent.receiver_refused",
    AgentSocketKilled => "agent.socket.killed",
    AgentSocketKillRefused => "agent.socket.kill_refused",
    AgentProcessKilled => "agent.process.killed",
    AgentProcessKillRefused => "agent.process.kill_refused",
    AgentAccountChanged => "agent.account.changed",
    AgentAccountChangeRefused => "agent.account.change_refused",
    AgentUnitControlled => "agent.unit.controlled",
    AgentUnitControlRefused => "agent.unit.control_refused",
    AgentCronChanged => "agent.cron.changed",
    AgentCronChangeRefused => "agent.cron.change_refused",
}

impl KnownKind {
    pub fn resolves(self) -> Option<KnownKind> {
        match self {
            KnownKind::PortListenRemoved => Some(KnownKind::PortListenNew),
            KnownKind::UserAccountRemoved => Some(KnownKind::UserAccountNew),
            KnownKind::UserSshkeyRemoved => Some(KnownKind::UserSshkeyAdded),
            KnownKind::PersistenceCronRemoved => Some(KnownKind::PersistenceCronNew),
            KnownKind::ContainerImageRemoved => Some(KnownKind::ContainerImageNew),
            KnownKind::ContainerVolumeRemoved => Some(KnownKind::ContainerVolumeNew),
            KnownKind::ContainerNetworkRemoved => Some(KnownKind::ContainerNetworkNew),
            KnownKind::ContainerProjectRemoved => Some(KnownKind::ContainerProjectNew),
            KnownKind::ContainerPodRemoved => Some(KnownKind::ContainerPodNew),
            KnownKind::ContainerSecretRemoved => Some(KnownKind::ContainerSecretNew),
            KnownKind::AgentBudgetRecovered => Some(KnownKind::AgentBudgetExceeded),
            KnownKind::AgentCollectorRecovered => Some(KnownKind::AgentCollectorDegraded),
            KnownKind::AgentBufferDrained => Some(KnownKind::AgentBufferDropping),
            KnownKind::FirewallEnabled => Some(KnownKind::FirewallDisabled),
            _ => None,
        }
    }
}
