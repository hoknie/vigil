use vigil_model::Change;

use super::{ResourceLimits, file_rules, launch_rules, process_rules, resource_rules};
use crate::services::findings_for;

pub(super) fn processes(change: &Change) -> Vec<(String, String)> {
    findings_for(process_rules(), std::slice::from_ref(change))
}

pub(super) fn launches(change: &Change) -> Vec<(String, String)> {
    findings_for(launch_rules(), std::slice::from_ref(change))
}

pub(super) fn files(change: &Change) -> Vec<(String, String)> {
    findings_for(file_rules(), std::slice::from_ref(change))
}

pub(super) fn resources(change: &Change) -> Vec<(String, String)> {
    findings_for(
        resource_rules(ResourceLimits::default()),
        std::slice::from_ref(change),
    )
}

pub(super) fn resources_tick(changes: &[Change]) -> Vec<(String, String)> {
    findings_for(resource_rules(ResourceLimits::default()), changes)
}
