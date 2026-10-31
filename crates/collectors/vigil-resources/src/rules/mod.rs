#[cfg(test)]
mod tests;
#[cfg(test)]
mod verdict;

mod clock_stepped;
mod cpu_saturated;
mod disk_low;
mod host_rebooted;
mod inodes_low;
mod memory_low;
mod resource_finding;
mod resource_limits;
mod set;

pub use clock_stepped::ClockStepped;
pub use cpu_saturated::CpuSaturated;
pub use disk_low::DiskLow;
pub use host_rebooted::HostRebooted;
pub use inodes_low::InodesLow;
pub use memory_low::MemoryLow;
pub use resource_limits::{
    CLOCK_SKEW_SECONDS, CPU_BUSY_PERCENT, DISK_FREE_PERCENT, INODE_FREE_PERCENT,
    MEMORY_AVAILABLE_PERCENT, ResourceLimits,
};
pub use set::resource_rules;
