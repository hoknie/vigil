#[cfg(test)]
mod fixture;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod verdict;

mod accounts;
mod firewall;
mod keys;
mod launches;
mod persistence;
mod processes;
mod sockets;

pub use accounts::{
    AccountUnlocked, NewAccount, PasswordChanged, PrivilegedGroupMemberAdded, RemovedAccount,
    SecondRootAccount, SudoGrantAdded, account_rules,
};
pub use firewall::{
    FirewallDisabled, FirewallEnabled, FirewallPolicyWeakened, FirewallRulesetFlushed,
    firewall_rules,
};
pub use keys::{SshKeyAdded, SshKeyRemoved};
pub use launches::{
    FirstLaunchForUser, LaunchFromWritablePath, LaunchedBinaryMissing, launch_rules,
};
pub use persistence::{
    KernelModuleLoaded, NewCronJob, NewTimer, NewUnit, PreloadChanged, ShellProfileChanged,
    UnitCommandChanged, persistence_rules,
};
pub use processes::{
    NewRootProcess, ProcessBinaryDeleted, ProcessFromWritablePath, UnexpectedParent, process_rules,
};
pub use sockets::{
    ClosedListeningPort, ExposedListeningPort, ListenFromWritablePath, ListeningBinaryDeleted,
    ListeningPortOwnerChanged, NewListeningPort, listening_port_rules,
};
