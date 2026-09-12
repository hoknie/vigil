use std::sync::MutexGuard;

use vigil_model::Snapshot;

use super::ResourcesCollector;
use super::files::{Refusal, UPTIME};
use super::source::TOLERANCE_SECONDS;
use crate::CollectError;
use crate::helpers::steadied;
use crate::parsers::{BOOT, ResourcesReading, booted_at_of, resources_snapshot};

impl ResourcesCollector {
    pub(super) fn reading(&self) -> Result<Snapshot, CollectError> {
        let boot_id = self.files.boot_id().ok();
        let (filesystems, _) = self.filesystems();
        let reading = ResourcesReading {
            boot_id: boot_id.as_deref(),
            booted_at: self.moment_this_host_booted(),
            memory: self.files.memory().ok(),
            filesystems: &filesystems,
        };

        match reading.says_nothing() {
            true => Err(self.nothing_was_read()),
            false => Ok(resources_snapshot(&(self.now)(), &reading)),
        }
    }

    pub(super) fn remember(&self, previous: &Snapshot) {
        let Some(booted_at) = previous.items.get(BOOT).and_then(booted_at_of) else {
            return;
        };

        *self.held() = Some(booted_at);
    }

    fn moment_this_host_booted(&self) -> Option<i64> {
        let uptime = self.files.uptime_seconds().ok()?;
        let read = (self.wall_clock)()?.checked_sub(i64::try_from(uptime).ok()?)?;

        let mut held = self.held();
        let published = steadied(*held, read, TOLERANCE_SECONDS);
        *held = Some(published);

        Some(published)
    }

    fn held(&self) -> MutexGuard<'_, Option<i64>> {
        self.booted_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn nothing_was_read(&self) -> CollectError {
        let shown = self.files.path(UPTIME).display().to_string();

        match self.files.uptime_seconds() {
            Err(Refusal::Denied) => CollectError::Denied(shown),
            Err(Refusal::Absent) => CollectError::Absent(shown),
            _ => CollectError::Unreadable(format!(
                "{shown} and the files beside it: none of what this collector reads was in a \
                 shape it knows"
            )),
        }
    }
}
