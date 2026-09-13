use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::persistence_rules;

pub(super) fn persistence(change: &Change) -> Vec<(String, String)> {
    findings_for(persistence_rules(), std::slice::from_ref(change))
}
