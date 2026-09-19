#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod elsewhere;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub use elsewhere::EnginesCollector;
#[cfg(target_os = "linux")]
pub use linux::EnginesCollector;
#[cfg(target_os = "macos")]
pub use macos::EnginesCollector;
