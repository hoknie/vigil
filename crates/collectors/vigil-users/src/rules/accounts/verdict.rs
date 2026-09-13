use vigil_model::Change;
use vigil_rules::findings_for;

use super::set::account_rules;

pub(super) fn accounts(change: &Change) -> Vec<(String, String)> {
    findings_for(account_rules(), std::slice::from_ref(change))
}
