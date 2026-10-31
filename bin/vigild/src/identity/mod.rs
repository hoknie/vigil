mod addresses;
mod fqdn;
mod hex;
mod hmac;
mod host;
mod install;
mod installed_on;
#[cfg(not(target_os = "macos"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod moved;

pub use addresses::addresses;
pub use host::describe;
#[cfg(test)]
pub use installed_on::Change;
pub use moved::Moved;
