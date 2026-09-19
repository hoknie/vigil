#[cfg(test)]
mod tests;

mod listening;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod open_descriptors;
mod proc_net;
mod proc_net_unix;
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod socket_information;

pub use listening::{ProcessOwner, SocketsReading, listening_snapshot};
#[cfg(target_os = "macos")]
pub use open_descriptors::socket_descriptors;
pub use proc_net::{Protocol, SocketRow, parse_net_table};
pub use proc_net_unix::{UnixSocketRow, parse_unix_table};
#[cfg(target_os = "macos")]
pub use socket_information::{Door, SOCKET_INFORMATION_BYTES, door_of};
