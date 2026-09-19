mod device;
mod mask;
mod places;

#[cfg(target_os = "linux")]
pub use device::device_numbers;
#[cfg(target_os = "macos")]
pub use device::macos_device_numbers;
pub use mask::{check_mask, holds_a_mask, is_a_mask, matches};
pub use places::lists_in;
