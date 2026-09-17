use rustix::fs::Mode;

#[cfg(target_os = "linux")]
pub fn bits(mode: u32) -> Mode {
    Mode::from_bits_truncate(mode)
}

#[cfg(not(target_os = "linux"))]
pub fn bits(mode: u32) -> Mode {
    Mode::from_bits_truncate(mode as rustix::fs::RawMode)
}
