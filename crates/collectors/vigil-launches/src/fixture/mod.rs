#[cfg(test)]
mod tests;

mod launches;
mod rows;

pub use launches::launches;
pub use rows::{launch, launch_of_a_missing_program};
