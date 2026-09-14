#[cfg(not(target_os = "linux"))]
mod elsewhere;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
pub use elsewhere::{running, still_running};
#[cfg(target_os = "linux")]
pub use linux::{ProcessesCollector, running, still_running};
