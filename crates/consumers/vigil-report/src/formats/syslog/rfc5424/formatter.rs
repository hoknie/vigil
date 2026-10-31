use super::super::SyslogFacility;

pub struct Rfc5424 {
    pub(super) facility: SyslogFacility,
    pub(super) app_name: String,
    pub(super) process_id: u32,
    pub(super) budget: usize,
}
