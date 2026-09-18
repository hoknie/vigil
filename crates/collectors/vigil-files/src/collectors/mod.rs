#[cfg(test)]
mod tests;

mod linux;
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod lists;

#[cfg(target_os = "linux")]
pub use linux::FilesCollector;
