mod answers;
mod footprint;
mod limitations;
mod opening;
mod records;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use vigil_model::{CollectorStatus, ReporterStatus, Silence, Snapshot};

use self::footprint::Footprint;
use super::Ring;
use crate::types::Startup;

pub struct State {
    startup: Startup,
    collectors: Vec<CollectorStatus>,
    reporters: Vec<ReporterStatus>,
    snapshots: BTreeMap<String, Snapshot>,
    findings: Ring,
    silence: Silence,
    footprint: Footprint,
}
