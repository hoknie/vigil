#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod elsewhere;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub use elsewhere::{ProcessesCollector, running, still_running};
#[cfg(target_os = "linux")]
pub use linux::{ProcessesCollector, running, still_running};
#[cfg(target_os = "macos")]
pub use macos::{ProcessesCollector, running, still_running};
