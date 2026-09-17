#[cfg(test)]
mod tests;

mod framed;
mod render;
mod report;

pub use framed::{Framed, drawing, framed};
pub use render::{height, render};
