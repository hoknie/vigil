mod account;
#[cfg(target_os = "macos")]
mod choosing;

#[cfg(all(test, target_os = "macos"))]
mod tests;

pub use account::Account;
#[cfg(target_os = "macos")]
pub use choosing::run_as;
