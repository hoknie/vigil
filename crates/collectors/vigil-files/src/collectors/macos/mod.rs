mod collector;
mod filesystems;
#[path = "../linux/health.rs"]
mod health;
#[path = "../linux/masks.rs"]
mod masks;
#[path = "../linux/plan.rs"]
mod plan;
#[path = "../linux/reading.rs"]
mod reading;
#[path = "../linux/source.rs"]
mod source;
#[path = "../linux/stat.rs"]
mod stat;
#[path = "../linux/walk.rs"]
mod walk;

#[cfg(test)]
mod tests;

pub use collector::FilesCollector;
