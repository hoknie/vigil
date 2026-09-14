#[cfg(test)]
mod tests;

mod ndjson;
#[cfg(unix)]
mod syslog;

pub use ndjson::NdjsonSink;
#[cfg(unix)]
pub use syslog::{LOCAL_SOCKET, SyslogSink};
