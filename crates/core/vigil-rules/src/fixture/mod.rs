mod files;
mod foreign;
mod launches;
mod processes;
mod resources;

pub use files::{watched_directory, watched_file, watched_file_absent};
pub use foreign::of_another_collector;
pub use launches::{launch, launch_of_a_missing_program};
pub use processes::{program, program_with_deleted_binary};
pub use resources::{boot, boot_unreadable, filesystem, memory};
