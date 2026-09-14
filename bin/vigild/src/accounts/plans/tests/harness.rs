use std::collections::BTreeMap;

use vigil_model::{AccountChange, Snapshot};

use crate::accounts::plans::plan;
use crate::accounts::step::Step;
use crate::accounts::utility::Utility;

pub const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

pub const OTHER: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRq ci@build";

pub fn disk(files: &[(&str, &str)]) -> BTreeMap<String, String> {
    files
        .iter()
        .map(|(path, text)| (path.to_string(), text.to_string()))
        .collect()
}

pub fn planned(change: AccountChange, reading: &Snapshot, files: &[(&str, &str)]) -> Vec<Step> {
    let disk = disk(files);
    plan(&change, reading, &|path, _| Ok(disk.get(path).cloned())).expect("a plan")
}

pub fn fingerprint(line: &str) -> String {
    vigil_users::parse_authorized_keys(line)[0]
        .fingerprint
        .clone()
}

pub fn arguments(steps: &[Step]) -> Vec<(Utility, Vec<String>)> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Run { utility, arguments } => Some((*utility, arguments.clone())),
            _ => None,
        })
        .collect()
}
