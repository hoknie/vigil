mod authorized_keys;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod directory;
mod group;
mod roster;
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod sessions;
mod shadow;
mod sudoers;

pub use authorized_keys::{AuthorizedKey, parse_authorized_keys};
#[cfg(target_os = "macos")]
pub use directory::{DirectoryAccount, merged_accounts, merged_groups};
#[cfg(target_os = "macos")]
pub use group::GroupEntry;
pub use group::{PRIVILEGED_GROUPS, parse_group};
pub use roster::{AccountsReading, UserKeyFile, accounts_snapshot};
pub use sessions::{LOGIND, Session, SessionSource, UTMP, merge_sessions};
#[cfg(target_os = "linux")]
pub use sessions::{is_session_file, parse_logind_session, parse_utmp};
#[cfg(target_os = "linux")]
pub use shadow::ShadowFacts;
pub use shadow::parse_shadow;
pub use sudoers::{SudoGrant, parse_sudoers};
