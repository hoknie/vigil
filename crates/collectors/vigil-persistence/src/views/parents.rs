use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;

use serde_json::Value;

use super::fields::strings;
use super::tree::{PULLED_IN_BY, PULLS_IN};
use crate::types::Kind;

const UNIT: &str = "unit|";

pub(super) fn parents(items: &BTreeMap<String, Value>, key: &str) -> usize {
    if Kind::of(key) != Kind::Unit {
        return 0;
    }
    let Some(item) = items.get(key) else {
        return 0;
    };
    let own = key.strip_prefix(UNIT);

    let mut found: BTreeSet<&str> = BTreeSet::new();
    for named in PULLED_IN_BY {
        for name in strings(item, named) {
            if Some(name) == own {
                continue;
            }
            if let Some((other, _)) = items.get_key_value(&format!("unit|{name}")) {
                found.insert(other.as_str());
            }
        }
    }
    let units = items
        .range::<str, _>((Bound::Included(UNIT), Bound::Unbounded))
        .take_while(|(other, _)| other.starts_with(UNIT));
    for (other, value) in units {
        if other == key || Kind::of(other) != Kind::Unit {
            continue;
        }
        let pulls_it_in = PULLS_IN.iter().any(|named| {
            strings(value, named)
                .into_iter()
                .any(|name| Some(name) == own)
        });
        if pulls_it_in {
            found.insert(other.as_str());
        }
    }
    found.len()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::tree::tree;
    use super::*;

    fn items(pairs: &[(&str, Value)]) -> BTreeMap<String, Value> {
        pairs
            .iter()
            .map(|(name, links)| {
                let mut item = json!({"name": "x", "type": "service", "readable": true});
                for (field, value) in links.as_object().expect("an object") {
                    item[field] = value.clone();
                }
                (format!("unit|{name}"), item)
            })
            .collect()
    }

    #[test]
    fn one_row_counts_the_units_that_pull_it_in_exactly_as_the_whole_tree_does() {
        let reading = items(&[
            ("multi-user.target", json!({"wants": ["graphical.target"]})),
            (
                "graphical.target",
                json!({"wants": ["multi-user.target", "nginx.service"]}),
            ),
            (
                "nginx.service",
                json!({"wanted_by": ["multi-user.target"], "part_of": ["graphical.target"]}),
            ),
            ("sockets.target", json!({"requires": ["nginx.service"]})),
            ("lonely.service", json!({"wants": ["gone.service"]})),
            (
                "self.service",
                json!({"wants": ["self.service"], "wanted_by": ["self.service"]}),
            ),
        ]);

        for placed in tree(&reading) {
            assert_eq!(
                parents(&reading, &placed.key),
                placed.parents,
                "{}: a row drawn on its own must say what the tree says, or the list and the \
                 tree disagree about the same unit",
                placed.key
            );
        }
    }

    #[test]
    fn a_row_that_is_not_a_unit_has_nothing_pulling_it_in() {
        let mut reading = items(&[("nginx.service", json!({"wants": ["backup.timer"]}))]);
        reading.insert("timer|backup.timer".into(), json!({"name": "backup.timer"}));

        assert_eq!(parents(&reading, "timer|backup.timer"), 0);
        assert_eq!(parents(&reading, "unit|not-in-the-reading.service"), 0);
    }
}
