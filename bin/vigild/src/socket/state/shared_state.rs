use std::collections::{BTreeMap, BTreeSet};

use vigil_model::{CollectorStatus, Finding, ReporterStatus, Silence, Snapshot};

use super::super::Ring;
use super::footprint::Footprint;
use crate::types::Startup;

pub struct State {
    pub(super) startup: Startup,
    pub(super) collectors: Vec<CollectorStatus>,
    pub(super) reporters: Vec<ReporterStatus>,
    pub(super) snapshots: BTreeMap<String, Snapshot>,
    pub(super) findings: Ring,
    pub(super) silence: Silence,
    pub(super) footprint: Footprint,
    pub(super) raised_by_the_console: Vec<Finding>,
    pub(super) readings_asked_for: BTreeSet<String>,
}
