mod container;
mod kind;
mod runtime_sockets;

pub use container::{ContainerView, Family};
pub use kind::Kind;
#[cfg_attr(not(target_os = "linux"), allow(unused_imports))]
pub use runtime_sockets::RUNTIME_SOCKETS;
pub use runtime_sockets::is_a_runtime_socket;
