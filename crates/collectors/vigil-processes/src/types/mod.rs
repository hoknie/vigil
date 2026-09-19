mod platform;
mod process;

#[cfg(any(target_os = "macos", test))]
pub use platform::MACOS;
pub use platform::{LINUX, Platform};
pub use process::ProcessView;
