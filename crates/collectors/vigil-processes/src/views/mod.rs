#[cfg(test)]
mod tests;

mod detail;
mod fields;
mod footer;
mod pane;
mod running;
mod section;

pub use pane::Running;
pub use section::WhatHasRunHere;
