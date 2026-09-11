use std::fs;

use crate::parsers::{KernelModule, parse_modules};

pub(super) fn read_modules() -> Option<Vec<KernelModule>> {
    fs::read_to_string("/proc/modules")
        .ok()
        .map(|text| parse_modules(&text))
}
