mod fields;
mod head;
mod line;
mod message;
mod structured_data;
mod text;

#[cfg(test)]
mod fixture;

use super::SyslogFacility;

pub struct Rfc5424 {
    facility: SyslogFacility,
    app_name: String,
    process_id: u32,
    budget: usize,
}
