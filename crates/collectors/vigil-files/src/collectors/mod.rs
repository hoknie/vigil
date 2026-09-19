#[cfg(test)]
mod tests;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod elsewhere;
#[cfg(target_os = "linux")]
mod linux;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod lists;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub use elsewhere::FilesCollector;
#[cfg(target_os = "linux")]
pub use linux::FilesCollector;
#[cfg(target_os = "macos")]
pub use macos::FilesCollector;
