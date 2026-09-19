#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod elsewhere;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub use elsewhere::UsersCollector;
#[cfg(target_os = "linux")]
pub use linux::UsersCollector;
#[cfg(target_os = "macos")]
pub use macos::UsersCollector;
