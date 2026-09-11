use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::kind::Kind;
use crate::ui::screens::programs::strings;

pub(super) struct Placed {
    pub key: String,
    pub depth: usize,
    pub parents: usize,
}

const PULLED_IN_BY: &[&str] = &["wanted_by", "required_by", "part_of"];

const PULLS_IN: &[&str] = &["wants", "requires"];

pub(super) fn tree(items: &BTreeMap<String, Value>) -> Vec<Placed> {
    let units: BTreeSet<&String> = items
        .keys()
        .filter(|key| Kind::of(key) == Kind::Unit)
        .collect();

    let mut children: BTreeMap<&String, BTreeSet<&String>> = BTreeMap::new();
    let mut parents: BTreeMap<&String, BTreeSet<&String>> = BTreeMap::new();
    let mut edges = 0usize;

    for key in &units {
        let item = &items[*key];
        for named in PULLED_IN_BY {
            for other in named_units(&units, item, named) {
                if other != *key {
                    children.entry(other).or_default().insert(key);
                    parents.entry(key).or_default().insert(other);
                    edges += 1;
                }
            }
        }
        for named in PULLS_IN {
            for other in named_units(&units, item, named) {
                if other != *key {
                    children.entry(key).or_default().insert(other);
                    parents.entry(other).or_default().insert(key);
                    edges += 1;
                }
            }
        }
    }

    let roots = units
        .iter()
        .copied()
        .filter(|key| !parents.contains_key(key))
        .chain(units.iter().copied());

    let ceiling = edges + units.len() + 1;
    let mut placed: BTreeSet<&String> = BTreeSet::new();
    let mut out: Vec<Placed> = Vec::with_capacity(units.len());
    let mut steps = 0usize;

    for root in roots {
        if placed.contains(root) {
            continue;
        }
        let mut stack: Vec<(&String, usize)> = vec![(root, 0)];
        while let Some((key, depth)) = stack.pop() {
            steps += 1;
            if steps > ceiling {
                return out;
            }
            if !placed.insert(key) {
                continue;
            }
            out.push(Placed {
                key: key.to_string(),
                depth,
                parents: parents.get(key).map(BTreeSet::len).unwrap_or_default(),
            });
            let Some(below) = children.get(key) else {
                continue;
            };
            for child in below.iter().rev() {
                if !placed.contains(*child) {
                    stack.push((child, depth + 1));
                }
            }
        }
    }

    out
}

fn named_units<'a>(units: &BTreeSet<&'a String>, item: &Value, named: &str) -> Vec<&'a String> {
    strings(item, named)
        .into_iter()
        .filter_map(|name| units.get(&format!("unit|{name}")).copied())
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn unit(links: Value) -> Value {
        let mut item = json!({"name": "x", "type": "service", "readable": true});
        for (key, value) in links.as_object().expect("an object") {
            item[key] = value.clone();
        }
        item
    }

    fn items(pairs: &[(&str, Value)]) -> BTreeMap<String, Value> {
        pairs
            .iter()
            .map(|(name, links)| (format!("unit|{name}"), unit(links.clone())))
            .collect()
    }

    fn drawn(placed: &[Placed]) -> Vec<String> {
        placed
            .iter()
            .map(|row| format!("{}{}", "  ".repeat(row.depth), row.key))
            .collect()
    }

    #[test]
    fn a_target_holds_what_the_files_say_it_pulls_in() {
        let placed = tree(&items(&[
            ("multi-user.target", json!({})),
            ("nginx.service", json!({"wanted_by": ["multi-user.target"]})),
            (
                "cron.service",
                json!({"required_by": ["multi-user.target"]}),
            ),
        ]));

        assert_eq!(
            drawn(&placed),
            vec![
                "unit|multi-user.target",
                "  unit|cron.service",
                "  unit|nginx.service",
            ]
        );
    }

    #[test]
    fn what_a_unit_wants_hangs_under_it_and_not_the_other_way_round() {
        let placed = tree(&items(&[
            ("a.service", json!({"wants": ["b.service"]})),
            ("b.service", json!({})),
        ]));

        assert_eq!(drawn(&placed), vec!["unit|a.service", "  unit|b.service"]);
    }

    #[test]
    fn a_unit_pulled_in_by_two_targets_is_drawn_once_and_counts_the_rest() {
        let placed = tree(&items(&[
            ("multi-user.target", json!({})),
            ("sockets.target", json!({})),
            (
                "nginx.service",
                json!({"wanted_by": ["multi-user.target", "sockets.target"]}),
            ),
        ]));

        assert_eq!(
            drawn(&placed),
            vec![
                "unit|multi-user.target",
                "  unit|nginx.service",
                "unit|sockets.target",
            ],
            "the first of its parents in order holds it, and the subtree is not copied"
        );
        let nginx = placed
            .iter()
            .find(|row| row.key == "unit|nginx.service")
            .expect("it is on the screen");
        assert_eq!(nginx.parents, 2, "and the row says how many pull it in");
    }

    #[test]
    fn a_ring_of_units_that_want_each_other_is_walked_once_and_not_for_ever() {
        let placed = tree(&items(&[
            ("a.service", json!({"wants": ["b.service"]})),
            ("b.service", json!({"wants": ["c.service"]})),
            ("c.service", json!({"wants": ["a.service"]})),
        ]));

        assert_eq!(
            drawn(&placed),
            vec!["unit|a.service", "  unit|b.service", "    unit|c.service",],
            "a ring with no way in is opened at its first unit rather than dropped"
        );
    }

    #[test]
    fn a_unit_that_nothing_pulls_in_is_a_root_of_its_own_and_does_not_disappear() {
        let placed = tree(&items(&[
            ("multi-user.target", json!({})),
            ("nginx.service", json!({"wanted_by": ["multi-user.target"]})),
            ("orphan.service", json!({})),
        ]));

        assert!(
            drawn(&placed).contains(&"unit|orphan.service".to_string()),
            "{:?}",
            drawn(&placed)
        );
    }

    #[test]
    fn the_tree_holds_every_unit_of_the_reading_exactly_once() {
        let reading = items(&[
            ("multi-user.target", json!({"wants": ["graphical.target"]})),
            ("graphical.target", json!({"wants": ["multi-user.target"]})),
            (
                "nginx.service",
                json!({"wanted_by": ["multi-user.target"], "part_of": ["graphical.target"]}),
            ),
            ("lonely.service", json!({"wants": ["gone.service"]})),
            ("self.service", json!({"wants": ["self.service"]})),
        ]);

        let placed = tree(&reading);

        let mut keys: Vec<&String> = placed.iter().map(|row| &row.key).collect();
        keys.sort();
        let mut expected: Vec<&String> = reading.keys().collect();
        expected.sort();
        assert_eq!(
            keys, expected,
            "a view that hides an object is a silent zero"
        );
    }

    #[test]
    fn a_name_no_unit_file_answers_to_is_not_a_row_and_not_a_parent() {
        let placed = tree(&items(&[(
            "nginx.service",
            json!({"wanted_by": ["multi-user.target"]}),
        )]));

        assert_eq!(drawn(&placed), vec!["unit|nginx.service"]);
        assert_eq!(placed[0].parents, 0);
    }

    #[test]
    fn a_timer_is_not_a_unit_row_so_it_is_neither_in_the_tree_nor_a_branch_of_it() {
        let mut reading = items(&[("nginx.service", json!({"wants": ["backup.timer"]}))]);
        reading.insert("timer|backup.timer".into(), json!({"name": "backup.timer"}));

        let placed = tree(&reading);

        assert_eq!(drawn(&placed), vec!["unit|nginx.service"]);
    }
}
