#[cfg(test)]
mod tests;

mod catalogue;
mod registry;

pub use catalogue::{every_seconds_of, is_known, names, subject_of, unit_of, watched};
pub use registry::modules;
