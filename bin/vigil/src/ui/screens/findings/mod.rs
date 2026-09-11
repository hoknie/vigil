#[cfg(test)]
mod tests;

mod columns;
mod notices;
mod regions;
mod render;
mod rows;
mod shape;
mod tally;

pub use render::render;
pub use rows::keys;
