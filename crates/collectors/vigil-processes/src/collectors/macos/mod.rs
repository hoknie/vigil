mod collector;
mod gathering;
mod health;
mod running;

#[cfg(test)]
mod tests;

pub use collector::ProcessesCollector;
pub use running::{running, still_running};
