use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::file_rules;

pub(super) fn files(change: &Change) -> Vec<(String, String)> {
    findings_for(file_rules(), std::slice::from_ref(change))
}
