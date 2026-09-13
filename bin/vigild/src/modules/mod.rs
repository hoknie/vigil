#[cfg(test)]
mod tests;

mod catalogue;
mod registry;

pub use catalogue::{every_seconds_of, is_known, names, subject_of, unit_of, watched};
#[cfg_attr(not(target_os = "linux"), allow(unused_imports))]
pub use registry::modules;
