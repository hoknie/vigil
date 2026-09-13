use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::container_rules;

pub(super) fn containers(change: &Change) -> Vec<(String, String)> {
    findings_for(container_rules(), std::slice::from_ref(change))
}
