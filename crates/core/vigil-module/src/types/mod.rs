#[cfg(test)]
mod tests;

mod clock;
mod settings;

pub use clock::Clock;
pub use settings::{Settings, SettingsError};
