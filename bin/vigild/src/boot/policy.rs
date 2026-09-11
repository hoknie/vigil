use crate::Config;
use crate::types::Policy;

pub fn of(config: &Config) -> Policy {
    let policy = Policy::new(config.suppressions.clone());
    if policy.suppression_count() > 0 {
        eprintln!(
            "  suppressions: {} from the configuration",
            policy.suppression_count()
        );
        for description in policy.describe_suppressions() {
            eprintln!("    {description}");
        }
    }
    policy
}
