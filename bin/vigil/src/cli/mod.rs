#[cfg(test)]
mod tests;

mod options;
mod style;

pub use options::{
    CAPTURE_MOVED, COLLECTOR_LIVES_IN_THE_DAEMON, CONFIGURE_LIVES_IN_THE_DAEMON, Cli, Command,
    Console, Opening, Silencing, the_old_shape,
};
