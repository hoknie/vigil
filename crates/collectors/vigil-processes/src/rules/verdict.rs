use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::process_rules;

pub(super) fn processes(change: &Change) -> Vec<(String, String)> {
    findings_for(process_rules(), std::slice::from_ref(change))
}
