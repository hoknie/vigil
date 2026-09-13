use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::firewall_rules;

pub(super) fn firewall(change: &Change) -> Vec<(String, String)> {
    findings_for(firewall_rules(), std::slice::from_ref(change))
}

pub(super) fn firewall_tick(changes: &[Change]) -> Vec<(String, String)> {
    findings_for(firewall_rules(), changes)
}
