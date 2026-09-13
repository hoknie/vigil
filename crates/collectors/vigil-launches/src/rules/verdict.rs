use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::launch_rules;

pub(super) fn launches(change: &Change) -> Vec<(String, String)> {
    findings_for(launch_rules(), std::slice::from_ref(change))
}
