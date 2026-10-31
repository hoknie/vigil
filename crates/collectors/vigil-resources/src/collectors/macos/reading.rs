use std::sync::MutexGuard;

use vigil_collect::{CollectError, steadied};
use vigil_model::Snapshot;

use super::ResourcesCollector;
use super::filesystems::filesystems;
use super::source::{BOOT_TIME, TOLERANCE_SECONDS, boot_session, memory, processors};
use crate::helpers::Sample;
use crate::parsers::{BOOT, ProcessorReading, ResourcesReading, booted_at_of, resources_snapshot};

impl ResourcesCollector {
    pub(super) fn reading(&self) -> Result<Snapshot, CollectError> {
        let boot_id = boot_session();
        let (filesystems, _) = filesystems();
        let reading = ResourcesReading {
            boot_id: boot_id.as_deref(),
            booted_at: self.moment_this_host_booted(),
            memory: memory(),
            processors: self.processors(),
            filesystems: &filesystems,
        };

        match reading.says_nothing() {
            true => Err(CollectError::Unreadable(format!(
                "{BOOT_TIME} and the values beside it: none of what this collector reads was \
                 in a shape it knows"
            ))),
            false => Ok(resources_snapshot(&(self.now)(), &reading)),
        }
    }

    fn processors(&self) -> Option<ProcessorReading> {
        let counters = processors()?;
        let busy = self
            .busy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(Sample {
                at_seconds: self.started.elapsed().as_secs(),
                total: counters.total,
                busy: counters.busy,
            });

        Some(ProcessorReading {
            processors: counters.processors,
            busy_percent_step: busy.percent_step,
            window_seconds: busy.window_seconds,
        })
    }

    pub(super) fn remember(&self, previous: &Snapshot) {
        let Some(booted_at) = previous.items.get(BOOT).and_then(booted_at_of) else {
            return;
        };

        *self.held() = Some(booted_at);
    }

    fn moment_this_host_booted(&self) -> Option<i64> {
        let read = (self.booted)()?;

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
}
