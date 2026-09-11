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

#[cfg(not(target_os = "linux"))]
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
            cfg!(target_os = "linux"),
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
    fn the_ceiling_is_the_one_the_product_states() {
        assert_eq!(CEILING_KB, 65_536, "64 MB, stated in megabytes");
    }
}
