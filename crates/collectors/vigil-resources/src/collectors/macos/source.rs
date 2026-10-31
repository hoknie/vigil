use vigil_collect::sysctl_named;

use crate::parsers::{
    MemoryFacts, ProcessorCounters, available_step, parse_boot_session, parse_boot_time,
    parse_memory_size,
};

pub(super) const NAME: &str = "resources";

pub(super) const TOLERANCE_SECONDS: i64 = 2;

pub(super) const BOOT_SESSION: &str = "kern.bootsessionuuid";

pub(super) const BOOT_TIME: &str = "kern.boottime";

pub(super) const MEMORY_SIZE: &str = "hw.memsize";

pub(super) const MEMORY_LEVEL: &str = "kern.memorystatus_level";

pub(super) const PROCESSORS: &str = "hw.logicalcpu";

pub(super) const PROCESSOR_LOAD: &str = "HOST_CPU_LOAD_INFO";

pub(super) fn boot_session() -> Option<String> {
    parse_boot_session(&sysctl_named(BOOT_SESSION).ok()?)
}

pub(super) fn boot_time() -> Option<i64> {
    parse_boot_time(&sysctl_named(BOOT_TIME).ok()?)
}

pub(super) fn memory() -> Option<MemoryFacts> {
    let mut memory = parse_memory_size(&sysctl_named(MEMORY_SIZE).ok()?)?;
    memory.available_percent_step = sysctl_named(MEMORY_LEVEL)
        .ok()
        .and_then(|bytes| integer(&bytes))
        .map(available_step);
    Some(memory)
}

pub(super) fn processors() -> Option<ProcessorCounters> {
    let processors = integer(&sysctl_named(PROCESSORS).ok()?)?;
    let ticks = processor_ticks()?;
    let total: u64 = ticks.iter().map(|tick| u64::from(*tick)).sum();
    let idle = u64::from(ticks[libc::CPU_STATE_IDLE as usize]);

    (processors > 0).then_some(ProcessorCounters {
        processors,
        total,
        busy: total.saturating_sub(idle),
    })
}

fn processor_ticks() -> Option<[u32; libc::CPU_STATE_MAX as usize]> {
    let mut load = libc::host_cpu_load_info {
        cpu_ticks: [0; libc::CPU_STATE_MAX as usize],
    };
    let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
    let answered = unsafe {
        libc::host_statistics(
            host_port(),
            libc::HOST_CPU_LOAD_INFO,
            (&mut load as *mut libc::host_cpu_load_info).cast(),
            &mut count,
        )
    };

    match answered {
        libc::KERN_SUCCESS => Some(load.cpu_ticks),
        _ => None,
    }
}

fn host_port() -> libc::mach_port_t {
    static HOST: std::sync::OnceLock<libc::mach_port_t> = std::sync::OnceLock::new();

    #[allow(deprecated)]
    *HOST.get_or_init(|| unsafe { libc::mach_host_self() })
}

fn integer(bytes: &[u8]) -> Option<u64> {
    match bytes.len() {
        4 => Some(u64::from(u32::from_ne_bytes(bytes.try_into().ok()?))),
        8 => Some(u64::from_ne_bytes(bytes.try_into().ok()?)),
        _ => None,
    }
}
