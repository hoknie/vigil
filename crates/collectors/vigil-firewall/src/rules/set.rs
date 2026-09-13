use super::{FirewallDisabled, FirewallEnabled, FirewallPolicyWeakened, FirewallRulesetFlushed};
use vigil_rules::RuleSet;

pub fn firewall_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(FirewallDisabled),
        Box::new(FirewallEnabled),
        Box::new(FirewallRulesetFlushed),
        Box::new(FirewallPolicyWeakened),
    ])
}
