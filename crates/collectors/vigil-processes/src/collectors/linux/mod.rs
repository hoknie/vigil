mod collector;
mod running;

pub use collector::ProcessesCollector;
pub use running::{running, still_running};
