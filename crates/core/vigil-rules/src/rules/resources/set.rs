use super::resource_limits::ResourceLimits;
use super::{ClockStepped, DiskLow, HostRebooted, InodesLow};
use crate::RuleSet;

pub fn resource_rules(limits: ResourceLimits) -> RuleSet {
    RuleSet::of(vec![
        Box::new(HostRebooted),
        Box::new(ClockStepped::watching(limits)),
        Box::new(DiskLow::watching(limits)),
        Box::new(InodesLow::watching(limits)),
    ])
}
