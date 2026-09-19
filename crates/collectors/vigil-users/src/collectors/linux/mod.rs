mod health;
mod keys;
mod reading;
mod sessions;
mod sudoers;

#[cfg(test)]
mod tests;

use vigil_model::{Rfc3339, Snapshot};

use vigil_collect::{CollectError, Collector, Health};

pub const PASSWD: &str = "/etc/passwd";
pub const GROUP: &str = "/etc/group";
pub const SHADOW: &str = "/etc/shadow";
pub const SUDOERS: &str = "/etc/sudoers";
pub const VENDOR_SUDOERS: &str = "/usr/etc/sudoers";
pub const SUDOERS_DIRECTORY: &str = "/etc/sudoers.d";

pub struct UsersCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
}

impl UsersCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        UsersCollector { now: Box::new(now) }
    }

    fn taken_at(&self) -> Rfc3339 {
        (self.now)()
    }
}

impl Collector for UsersCollector {
    fn name(&self) -> &'static str {
        "users"
    }

    fn available(&self) -> Health {
        health::health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        reading::reading(&self.taken_at())
    }
}
