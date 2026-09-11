#[cfg(test)]
mod tests;

mod options;
mod style;

pub use options::{
    CAPTURE_MOVED, CONFIGURE_LIVES_IN_THE_DAEMON, Cli, Command, Console, Opening, the_old_shape,
};
