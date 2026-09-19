#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn device_numbers(device: u64) -> (u32, u32) {
    let major = ((device >> 8) & 0xfff) | ((device >> 32) & !0xfff);
    let minor = (device & 0xff) | ((device >> 12) & !0xff);
    (
        u32::try_from(major).unwrap_or(u32::MAX),
        u32::try_from(minor).unwrap_or(u32::MAX),
    )
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn macos_device_numbers(device: u64) -> (u32, u32) {
    let major = (device >> 24) & 0xff;
    let minor = device & 0x00ff_ffff;
    (
        u32::try_from(major).unwrap_or(u32::MAX),
        u32::try_from(minor).unwrap_or(u32::MAX),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_device_number_splits_into_the_major_and_minor_the_mount_table_prints() {
        assert_eq!(device_numbers(0x0801), (8, 1));
        assert_eq!(device_numbers(0x0000_0000_0000_0023), (0, 35));
        assert_eq!(device_numbers((259 << 8) | 3), (259, 3));
        assert_eq!(
            device_numbers(0x0010_082c),
            (8, 300),
            "a minor past 255 is split across the number, and reading only the low byte would \
             put two disks on one mount"
        );
    }

    #[test]
    fn a_device_number_of_macos_splits_its_major_off_the_top_byte() {
        assert_eq!(macos_device_numbers(0x0100_0010), (1, 16));
        assert_eq!(macos_device_numbers(0x1300_0005), (19, 5));
        assert_eq!(
            macos_device_numbers(0xffff_ffff_8100_0002),
            (0x81, 2),
            "a number the standard library widened with its sign is still the device it was"
        );
        assert_ne!(
            macos_device_numbers(0x0100_0010),
            device_numbers(0x0100_0010),
            "the two kernels pack the same two numbers differently"
        );
    }
}
