#[cfg(test)]
mod tests;
#[cfg(test)]
mod verdict;

mod new_root_process;
mod process_binary_deleted;
mod process_finding;
mod process_from_writable_path;
mod set;
mod unexpected_parent;

pub use new_root_process::NewRootProcess;
pub use process_binary_deleted::ProcessBinaryDeleted;
pub use process_from_writable_path::ProcessFromWritablePath;
pub use set::process_rules;
pub use unexpected_parent::UnexpectedParent;
