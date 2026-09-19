mod hex;
mod hmac;
mod host;
mod install;
#[cfg(not(target_os = "macos"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

pub use host::describe;
