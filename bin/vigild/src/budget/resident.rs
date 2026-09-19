pub const CEILING_KB: u64 = 64 * 1_024;

#[cfg(target_os = "linux")]
pub fn kilobytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|number| number.parse().ok())
}

#[cfg(target_os = "macos")]
pub fn kilobytes() -> Option<u64> {
    const BYTES_IN_A_KILOBYTE: u64 = 1_024;

    let mut task = std::mem::MaybeUninit::<libc::proc_taskinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_taskinfo>() as libc::c_int;
    let written = unsafe {
        libc::proc_pidinfo(
            std::process::id() as libc::c_int,
            libc::PROC_PIDTASKINFO,
            0,
            task.as_mut_ptr().cast(),
            size,
        )
    };
    if written != size {
        return None;
    }
    let task = unsafe { task.assume_init() };
    Some(task.pti_resident_size / BYTES_IN_A_KILOBYTE)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn kilobytes() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_platform_that_cannot_measure_its_own_memory_says_so_instead_of_reporting_zero() {
        let measured = kilobytes();

        assert_eq!(
            measured.is_some(),
            cfg!(any(target_os = "linux", target_os = "macos")),
            "a build that cannot read its own resident size answers with nothing at all"
        );
        if let Some(resident_kb) = measured {
            assert!(
                resident_kb > 0,
                "a resident size of zero is not a measurement of a running process"
            );
        }
    }

    #[test]
    fn what_is_measured_is_the_resident_size_now_and_it_follows_what_the_process_holds() {
        let Some(before) = kilobytes() else {
            return;
        };
        let held = std::hint::black_box(vec![1u8; 32 * 1_024 * 1_024]);

        let after = kilobytes().expect("measured once, measured again");

        assert!(
            after >= before + 16 * 1_024,
            "a peak such as getrusage's ru_maxrss never comes back down under the ceiling, so \
             a crossing could never close: {before} kB then {after} kB with {} bytes held",
            held.len()
        );
    }

    #[test]
    fn the_ceiling_is_the_one_the_product_states() {
        assert_eq!(CEILING_KB, 65_536, "64 MB, stated in megabytes");
    }
}
