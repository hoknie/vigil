use std::fmt;

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::clock::Clock;

#[derive(Debug, Clone)]
pub struct Settings {
    now: Clock,
    key: Option<&'static str>,
    said: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsError {
    pub key: String,
    pub why: String,
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in the configuration file: {}", self.key, self.why)
    }
}

impl std::error::Error for SettingsError {}

impl Settings {
    pub fn plain(now: Clock) -> Settings {
        Settings {
            now,
            key: None,
            said: Value::Null,
        }
    }

    pub fn of(now: Clock, key: &'static str, said: Value) -> Settings {
        Settings {
            now,
            key: Some(key),
            said,
        }
    }

    pub fn now(&self) -> Clock {
        self.now
    }

    pub fn said(&self) -> &Value {
        &self.said
    }

    pub fn read<T: DeserializeOwned + Default>(&self) -> Result<T, SettingsError> {
        if self.said.is_null() {
            return Ok(T::default());
        }
        serde_json::from_value(self.said.clone()).map_err(|error| SettingsError {
            key: self.key.unwrap_or("this module").to_string(),
            why: error.to_string(),
        })
    }
}
