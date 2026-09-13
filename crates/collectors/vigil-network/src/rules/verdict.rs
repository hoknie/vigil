use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::listening_port_rules;

pub(super) fn ports(change: &Change) -> Vec<(String, String)> {
    findings_for(listening_port_rules(), std::slice::from_ref(change))
}

pub(super) fn ports_tick(changes: &[Change]) -> Vec<(String, String)> {
    findings_for(listening_port_rules(), changes)
}
