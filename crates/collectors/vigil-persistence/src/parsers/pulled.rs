use std::collections::{BTreeMap, BTreeSet};

use super::entries::UnitFile;

pub(super) fn pulled_in_by(units: &[UnitFile]) -> BTreeMap<&str, BTreeSet<&str>> {
    let named: BTreeMap<&str, &UnitFile> = units
        .iter()
        .filter(|unit| unit.kind() != "timer")
        .map(|unit| (unit.name.as_str(), unit))
        .collect();

    let mut pulled: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (name, unit) in &named {
        let links = &unit.facts.links;
        for other in links.wants.iter().chain(&links.requires) {
            if other == name {
                continue;
            }
            if let Some((other, _)) = named.get_key_value(other.as_str()) {
                pulled.entry(other).or_default().insert(name);
            }
        }
    }
    pulled
}
