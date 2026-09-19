mod logind;
mod merge;
mod session;
mod source;
mod utmp;

#[cfg(test)]
mod tests;

#[cfg(any(target_os = "linux", test))]
pub use logind::{is_session_file, parse_logind_session};
pub use merge::merge_sessions;
pub use session::{LOGIND, Session, UTMP};
pub use source::SessionSource;
#[cfg(any(target_os = "linux", test))]
pub use utmp::parse_utmp;
