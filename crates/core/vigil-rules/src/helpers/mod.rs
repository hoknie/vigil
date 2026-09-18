mod host_paths;
mod writable;

pub use host_paths::{is_a_path_of_this_host, is_a_path_of_this_host_whatever_follows};
pub use writable::is_writable_path;
