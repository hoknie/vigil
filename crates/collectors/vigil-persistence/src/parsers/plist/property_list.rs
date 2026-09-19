use super::refusal::PlistRefusal;
use super::value::PlistValue;
use super::{binary, xml};

const BINARY: &[u8] = b"bplist00";

pub const DEEPEST: usize = 64;

pub fn parse_plist(bytes: &[u8]) -> Result<PlistValue, PlistRefusal> {
    match bytes.starts_with(BINARY) {
        true => binary::parse(bytes),
        false => xml::parse(bytes),
    }
}
