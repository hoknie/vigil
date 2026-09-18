pub fn device_numbers(device: u64) -> (u32, u32) {
    let major = ((device >> 8) & 0xfff) | ((device >> 32) & !0xfff);
    let minor = (device & 0xff) | ((device >> 12) & !0xff);
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
}
