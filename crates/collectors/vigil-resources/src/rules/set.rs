use super::resource_limits::ResourceLimits;
use super::{ClockStepped, CpuSaturated, DiskLow, HostRebooted, InodesLow, MemoryLow};
use vigil_rules::RuleSet;

pub fn resource_rules(limits: ResourceLimits) -> RuleSet {
    RuleSet::of(vec![
        Box::new(HostRebooted),
        Box::new(ClockStepped::watching(limits)),
        Box::new(DiskLow::watching(limits)),
        Box::new(InodesLow::watching(limits)),
        Box::new(CpuSaturated::watching(limits)),
        Box::new(MemoryLow::watching(limits)),
    ])
}
