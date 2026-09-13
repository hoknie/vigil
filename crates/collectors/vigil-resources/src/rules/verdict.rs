use vigil_model::Change;
use vigil_rules::findings_for;

use super::resource_limits::ResourceLimits;
use super::set::resource_rules;

pub(super) fn resources(change: &Change) -> Vec<(String, String)> {
    findings_for(
        resource_rules(ResourceLimits::default()),
        std::slice::from_ref(change),
    )
}

pub(super) fn resources_tick(changes: &[Change]) -> Vec<(String, String)> {
    findings_for(resource_rules(ResourceLimits::default()), changes)
}
