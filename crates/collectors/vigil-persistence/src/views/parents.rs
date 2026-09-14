use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;

use serde_json::Value;

use super::fields::strings;
use super::tree::{PULLED_IN_BY, PULLS_IN};
use crate::types::Kind;

const UNIT: &str = "unit|";

const CARRIED: &str = "pulled_in_by";

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
        looked_up(items, item, named, own, &mut found);
    }
    match item.get(CARRIED).is_some_and(Value::is_array) {
        true => looked_up(items, item, CARRIED, own, &mut found),
        false => walked(items, key, own, &mut found),
    }
    found.len()
}

fn looked_up<'a>(
    items: &'a BTreeMap<String, Value>,
    item: &Value,
    named: &str,
    own: Option<&str>,
    found: &mut BTreeSet<&'a str>,
) {
    for name in strings(item, named) {
        if Some(name) == own {
            continue;
        }
        if let Some((other, _)) = items.get_key_value(&format!("{UNIT}{name}")) {
            found.insert(other.as_str());
        }
    }
}

fn walked<'a>(
    items: &'a BTreeMap<String, Value>,
    key: &str,
    own: Option<&str>,
    found: &mut BTreeSet<&'a str>,
) {
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
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::tree::tree;
    use super::*;
    use crate::parsers::{
        PersistenceReading, PreloadFile, UnitFile, parse_unit, persistence_snapshot,
    };

    const FILES: &[(&str, &str)] = &[
        (
            "multi-user.target",
            "[Unit]\nWants=graphical.target rescue.service\n",
        ),
        (
            "graphical.target",
            "[Unit]\nWants=multi-user.target nginx.service\n",
        ),
        (
            "nginx.service",
            "[Unit]\nRequires=network.target\nPartOf=graphical.target\n\
             [Install]\nWantedBy=multi-user.target\n",
        ),
        ("sockets.target", "[Unit]\nRequires=nginx.service\n"),
        ("lonely.service", "[Unit]\nWants=gone.service\n"),
        (
            "self.service",
            "[Unit]\nWants=self.service\n[Install]\nWantedBy=self.service\n",
        ),
        ("rescue.service", "[Service]\nExecStart=/bin/sh\n"),
        ("backup.timer", "[Unit]\nWants=nginx.service\n"),
    ];

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

    fn built() -> BTreeMap<String, Value> {
        let units: Vec<UnitFile> = FILES
            .iter()
            .map(|(name, text)| UnitFile {
                name: name.to_string(),
                path: format!("/etc/systemd/system/{name}"),
                readable: true,
                facts: parse_unit(text),
            })
            .collect();
        let preload = PreloadFile {
            path: "/etc/ld.so.preload".into(),
            present: false,
            readable: true,
            entries: Vec::new(),
            digest: None,
        };
        let reading = PersistenceReading {
            units: &units,
            cron: &[],
            modules: Some(&[]),
            scripts: &[],
            preload: &preload,
        };
        persistence_snapshot("2026-09-14T12:00:00.000Z", &reading).items
    }

    fn agrees_with_the_tree(reading: &BTreeMap<String, Value>) {
        for placed in tree(reading) {
            assert_eq!(
                parents(reading, &placed.key),
                placed.parents,
                "{}: a row drawn on its own must say what the tree says, or the list and the \
                 tree disagree about the same unit",
                placed.key
            );
        }
    }

    #[test]
    fn one_row_counts_the_units_that_pull_it_in_exactly_as_the_whole_tree_does() {
        agrees_with_the_tree(&items(&[
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
        ]));
    }

    #[test]
    fn a_row_of_a_reading_the_collector_built_counts_its_parents_exactly_as_the_tree_does() {
        let reading = built();
        for (key, item) in reading.iter().filter(|(key, _)| key.starts_with(UNIT)) {
            assert!(
                item.get(CARRIED).is_some_and(Value::is_array),
                "{key}: the collector writes the reverse link under the name this view reads, \
                 or every row falls back to walking the units"
            );
        }
        assert!(
            tree(&reading).iter().any(|placed| placed.parents > 1),
            "the sample must hold a unit pulled in from both sides to prove anything"
        );

        agrees_with_the_tree(&reading);
    }

    #[test]
    fn a_row_of_a_reading_from_an_older_agent_without_the_reverse_link_still_counts_as_the_tree_does()
     {
        let mut reading = built();
        for item in reading.values_mut() {
            if let Some(object) = item.as_object_mut() {
                object.remove(CARRIED);
            }
        }

        agrees_with_the_tree(&reading);
    }

    #[test]
    fn a_carried_reverse_link_is_looked_up_by_key_and_the_other_units_are_not_walked() {
        let reading = items(&[
            (
                "a.service",
                json!({"pulled_in_by": ["b.service", "gone.service", "a.service"]}),
            ),
            ("b.service", json!({})),
            ("c.service", json!({"wants": ["a.service"]})),
        ]);

        assert_eq!(
            parents(&reading, "unit|a.service"),
            1,
            "the list the collector wrote is the answer: a name with no unit is not a parent, \
             the unit itself is not, and a unit the list does not name is never looked at"
        );
    }

    #[test]
    fn a_row_that_is_not_a_unit_has_nothing_pulling_it_in() {
        let mut reading = items(&[("nginx.service", json!({"wants": ["backup.timer"]}))]);
        reading.insert("timer|backup.timer".into(), json!({"name": "backup.timer"}));

        assert_eq!(parents(&reading, "timer|backup.timer"), 0);
        assert_eq!(parents(&reading, "unit|not-in-the-reading.service"), 0);
    }
}
