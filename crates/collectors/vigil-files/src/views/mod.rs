#[cfg(test)]
mod tests;

mod fields;
mod footer;
mod form;
mod notices;
mod pane;
mod section;
mod tally;

pub use form::{asked_for, path_watched};
pub use section::TheHostAndItsFiles;
