use crate::helpers::quoting::unquoted;
use crate::types::entry::Entry;

const HEADING: &str = "suppressions:";

const EMPTY: &str = "suppressions: []";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Changed { text: String, entries: usize },
    AlreadySo,
    NotOurs(String),
}

pub fn add(text: &str, entries: &[Entry]) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let (heading, held) = match shape(&lines) {
        Shape::NotOurs => return Edit::NotOurs(shape_we_do_not_edit()),
        Shape::Missing => (None, Vec::new()),
        Shape::Empty(at) => (Some(at), Vec::new()),
        Shape::Block(at) => (Some(at), items(&lines, at)),
    };

    let mut named: Vec<Named> = held.iter().filter_map(Item::named).collect();
    let mut written: Vec<String> = Vec::new();
    let mut wrote = 0;
    for entry in entries {
        let asked = Named {
            key: entry.key.clone(),
            prefix: entry.prefix,
            kind: entry.kind.clone(),
        };
        if named.contains(&asked) {
            continue;
        }
        named.push(asked);
        written.extend(entry.lines());
        wrote += 1;
    }
    if written.is_empty() {
        return Edit::AlreadySo;
    }

    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();
    match heading {
        None => {
            while out.last().is_some_and(|line| line.trim().is_empty()) {
                out.pop();
            }
            out.push(String::new());
            out.push(HEADING.to_string());
            out.extend(written);
        }
        Some(at) => {
            out[at] = HEADING.to_string();
            let after = held.last().map(|item| item.end).unwrap_or(at + 1);
            for (moved, line) in written.into_iter().enumerate() {
                out.insert(after + moved, line);
            }
        }
    }
    Edit::Changed {
        text: joined(&out),
        entries: wrote,
    }
}

pub fn remove(text: &str, keys: &[String]) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let heading = match shape(&lines) {
        Shape::NotOurs => return Edit::NotOurs(shape_we_do_not_edit()),
        Shape::Missing | Shape::Empty(_) => return Edit::AlreadySo,
        Shape::Block(at) => at,
    };

    let held = items(&lines, heading);
    let doomed: Vec<&Item> = held
        .iter()
        .filter(|item| {
            item.key
                .as_ref()
                .is_some_and(|(key, _)| keys.iter().any(|wanted| wanted == key))
        })
        .collect();
    if doomed.is_empty() {
        return Edit::AlreadySo;
    }

    let mut out: Vec<String> = lines
        .iter()
        .enumerate()
        .filter(|(at, _)| !doomed.iter().any(|item| item.holds(*at)))
        .map(|(_, line)| (*line).to_string())
        .collect();
    if held.len() == doomed.len() {
        out[heading] = EMPTY.to_string();
    }
    Edit::Changed {
        text: joined(&out),
        entries: doomed.len(),
    }
}

pub fn named(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    match shape(&lines) {
        Shape::Block(at) => items(&lines, at)
            .into_iter()
            .filter_map(|item| item.key.map(|(key, _)| key))
            .collect(),
        _ => Vec::new(),
    }
}

enum Shape {
    Missing,
    Empty(usize),
    Block(usize),
    NotOurs,
}

struct Item {
    start: usize,
    end: usize,
    key: Option<(String, bool)>,
    kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Named {
    key: String,
    prefix: bool,
    kind: Option<String>,
}

impl Item {
    fn holds(&self, line: usize) -> bool {
        line >= self.start && line < self.end
    }

    fn named(&self) -> Option<Named> {
        let (key, prefix) = self.key.clone()?;

        Some(Named {
            key,
            prefix,
            kind: self.kind.clone(),
        })
    }
}

fn shape(lines: &[&str]) -> Shape {
    for (at, line) in lines.iter().enumerate() {
        if !line.starts_with(HEADING) {
            continue;
        }
        return match line[HEADING.len()..].trim() {
            "" => Shape::Block(at),
            "[]" => Shape::Empty(at),
            _ => Shape::NotOurs,
        };
    }
    Shape::Missing
}

fn items(lines: &[&str], heading: usize) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();

    for (at, line) in lines.iter().enumerate().skip(heading + 1) {
        let held = line.trim();
        if held.is_empty() || held.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        match held.strip_prefix('-') {
            Some(first) => items.push(Item {
                start: at,
                end: at + 1,
                key: keyed(first.trim()),
                kind: kinded(first.trim()),
            }),
            None => {
                if let Some(item) = items.last_mut() {
                    item.end = at + 1;
                    if item.key.is_none() {
                        item.key = keyed(held);
                    }
                    if item.kind.is_none() {
                        item.kind = kinded(held);
                    }
                }
            }
        }
    }
    items
}

fn kinded(field: &str) -> Option<String> {
    field
        .strip_prefix("kind:")
        .map(|value| unquoted(value.trim()))
}

fn keyed(field: &str) -> Option<(String, bool)> {
    for (name, prefix) in [("finding_key_prefix:", true), ("finding_key:", false)] {
        if let Some(value) = field.strip_prefix(name) {
            return Some((unquoted(value.trim()), prefix));
        }
    }
    None
}

fn joined(lines: &[String]) -> String {
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn shape_we_do_not_edit() -> String {
    format!(
        "the {HEADING} in this file is not written as a block of lines, and this command edits \
         no shape it did not write. Add the entry by hand, or write the file again with `vigild \
         configure --force`"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = "\
state_dir: /var/lib/vigil
retention_days: 90

collectors:
  - ports

# What this host is expected to do.
suppressions: []

reporters: []
";

    fn entry(key: &str) -> Entry {
        Entry {
            key: key.into(),
            prefix: false,
            kind: None,
            until: None,
            reason: "the staging api, expected here".into(),
        }
    }

    fn changed(edit: Edit) -> String {
        match edit {
            Edit::Changed { text, .. } => text,
            other => panic!("{other:?}"),
        }
    }

    fn wrote(edit: Edit) -> usize {
        match edit {
            Edit::Changed { entries, .. } => entries,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_first_entry_turns_the_empty_list_into_a_block_and_moves_nothing_else() {
        let after = changed(add(SHIPPED, &[entry("port.listen|tcp|0.0.0.0:8080")]));

        assert!(after.contains("suppressions:\n"), "{after}");
        assert!(!after.contains("suppressions: []"), "{after}");
        assert!(
            after.contains("  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"\n"),
            "{after}"
        );
        assert!(
            after.contains("    reason: \"the staging api, expected here\"\n"),
            "{after}"
        );
        for kept in [
            "state_dir: /var/lib/vigil",
            "retention_days: 90",
            "  - ports",
            "# What this host is expected to do.",
            "reporters: []",
        ] {
            assert!(after.contains(kept), "{kept} was lost:\n{after}");
        }
    }

    #[test]
    fn a_second_entry_goes_under_the_first_and_not_through_it() {
        let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

        let twice = changed(add(&once, &[entry("port.listen|tcp|0.0.0.0:8080")]));

        let docker = twice.find("user|group|docker").expect("the first is there");
        let port = twice.find("0.0.0.0:8080").expect("the second is there");
        assert!(docker < port, "{twice}");
        assert!(twice.contains("reporters: []"), "{twice}");
        assert_eq!(
            twice.matches("  - finding_key:").count(),
            2,
            "one entry per key and no more:\n{twice}"
        );
    }

    #[test]
    fn what_is_already_written_down_is_not_written_down_twice() {
        let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

        assert_eq!(add(&once, &[entry("user|group|docker")]), Edit::AlreadySo);
    }

    #[test]
    fn one_key_of_two_that_is_new_is_written_and_the_one_that_is_there_is_left_alone() {
        let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

        let after = changed(add(
            &once,
            &[entry("user|group|docker"), entry("user|group|sudo")],
        ));

        assert_eq!(after.matches("user|group|docker").count(), 1, "{after}");
        assert!(after.contains("user|group|sudo"), "{after}");
    }

    #[test]
    fn one_key_named_twice_in_one_call_is_written_down_once() {
        let after = changed(add(
            SHIPPED,
            &[entry("user|group|docker"), entry("user|group|docker")],
        ));

        assert_eq!(
            after.matches("user|group|docker").count(),
            1,
            "the same object asked for twice is one entry, and a file with it twice is a file \
             an operator has to read twice to learn one thing:\n{after}"
        );
        assert_eq!(
            wrote(add(SHIPPED, &[entry("a|b"), entry("a|b"), entry("c|d")])),
            2
        );
    }

    #[test]
    fn the_same_object_narrowed_to_two_kinds_is_two_entries_because_it_is_two_statements() {
        let one = changed(add(
            SHIPPED,
            &[Entry {
                kind: Some("port.listen.new".into()),
                ..entry("port.listen|tcp|0.0.0.0:8080")
            }],
        ));

        let both = changed(add(
            &one,
            &[Entry {
                kind: Some("port.listen.removed".into()),
                ..entry("port.listen|tcp|0.0.0.0:8080")
            }],
        ));

        assert_eq!(
            both.matches("  - finding_key:").count(),
            2,
            "silencing 'the port appeared' is not silencing 'the port is gone', and a command \
             that called the second one a duplicate would refuse to write what was asked:\n{both}"
        );
        assert_eq!(
            add(
                &both,
                &[Entry {
                    kind: Some("port.listen.new".into()),
                    ..entry("port.listen|tcp|0.0.0.0:8080")
                }]
            ),
            Edit::AlreadySo,
            "and the same object with the same kind is still one entry"
        );
    }

    #[test]
    fn an_entry_written_by_hand_with_a_kind_is_read_with_it_and_not_taken_for_a_broader_one() {
        let by_hand = "\
suppressions:
  - finding_key: \"user|group|docker\"
    kind: user.group.privileged_member_added
    reason: the deploy user belongs there
";

        assert_eq!(
            add(by_hand, &[entry("user|group|docker")]),
            Edit::Changed {
                text: changed(add(by_hand, &[entry("user|group|docker")])),
                entries: 1,
            },
            "an entry that silences one kind does not silence the object, so the broader one \
             is a new entry and not a duplicate"
        );
    }

    #[test]
    fn a_prefix_and_an_exact_key_are_two_different_entries_and_neither_hides_the_other() {
        let exact = changed(add(SHIPPED, &[entry("port.listen|tcp|10.0.0.5:")]));
        let both = changed(add(
            &exact,
            &[Entry {
                prefix: true,
                ..entry("port.listen|tcp|10.0.0.5:")
            }],
        ));

        assert!(both.contains("  - finding_key: \"port.listen|tcp|10.0.0.5:\""));
        assert!(both.contains("  - finding_key_prefix: \"port.listen|tcp|10.0.0.5:\""));
    }

    #[test]
    fn a_kind_and_a_date_are_written_beside_the_key_that_carries_them() {
        let after = changed(add(
            SHIPPED,
            &[Entry {
                kind: Some("port.listen.new".into()),
                until: Some("2026-12-31T00:00:00.000Z".into()),
                ..entry("port.listen|tcp|0.0.0.0:8080")
            }],
        ));

        assert!(after.contains("    kind: \"port.listen.new\"\n"), "{after}");
        assert!(
            after.contains("    until: \"2026-12-31T00:00:00.000Z\"\n"),
            "{after}"
        );
    }

    #[test]
    fn a_file_that_never_had_the_key_gets_a_block_of_its_own_at_the_end() {
        let bare = "state_dir: /var/lib/vigil\n";

        let after = changed(add(bare, &[entry("user|group|docker")]));

        assert!(after.starts_with("state_dir: /var/lib/vigil\n"), "{after}");
        assert!(
            after.contains("\nsuppressions:\n  - finding_key:"),
            "{after}"
        );
    }

    #[test]
    fn a_shape_this_command_did_not_write_is_left_alone_and_named() {
        let inline = "suppressions: [{finding_key: a, reason: b}]\n";

        match add(inline, &[entry("user|group|docker")]) {
            Edit::NotOurs(said) => assert!(said.contains("by hand"), "{said}"),
            other => panic!("a file written another way must not be rewritten: {other:?}"),
        }
    }

    #[test]
    fn what_was_written_is_taken_out_again_and_the_file_is_the_one_it_started_as() {
        let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

        let after = changed(remove(&once, &["user|group|docker".to_string()]));

        assert_eq!(
            after, SHIPPED,
            "what add put in, remove takes out, down to the empty list:\n{after}"
        );
    }

    #[test]
    fn taking_one_of_two_out_leaves_the_other_where_it_was() {
        let both = changed(add(
            SHIPPED,
            &[entry("user|group|docker"), entry("user|group|sudo")],
        ));

        let after = changed(remove(&both, &["user|group|docker".to_string()]));

        assert!(!after.contains("docker"), "{after}");
        assert!(after.contains("user|group|sudo"), "{after}");
        assert!(after.contains("suppressions:\n"), "{after}");
        assert!(!after.contains("suppressions: []"), "{after}");
    }

    #[test]
    fn taking_out_what_is_not_there_changes_nothing_at_all() {
        assert_eq!(
            remove(SHIPPED, &["user|group|docker".to_string()]),
            Edit::AlreadySo
        );
        let once = changed(add(SHIPPED, &[entry("user|group|docker")]));
        assert_eq!(
            remove(&once, &["user|group|sudo".to_string()]),
            Edit::AlreadySo
        );
    }

    #[test]
    fn a_comment_somebody_wrote_between_two_entries_is_not_carried_off_with_one_of_them() {
        let by_hand = "\
suppressions:
  - finding_key: \"user|group|docker\"
    reason: the deploy user belongs there
  # the staging api moves about
  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"
    reason: staging
";

        let after = changed(remove(by_hand, &["user|group|docker".to_string()]));

        assert!(
            after.contains("# the staging api moves about"),
            "a line this command did not write must survive it: {after}"
        );
        assert!(after.contains("0.0.0.0:8080"), "{after}");
        assert!(!after.contains("docker"), "{after}");
    }

    #[test]
    fn an_entry_written_by_hand_without_quotes_is_still_found_by_its_key() {
        let by_hand = "suppressions:\n  - finding_key: user|group|docker\n    reason: ours\n";

        assert_eq!(named(by_hand), vec!["user|group|docker".to_string()]);
        assert_eq!(add(by_hand, &[entry("user|group|docker")]), Edit::AlreadySo);
        assert!(matches!(
            remove(by_hand, &["user|group|docker".to_string()]),
            Edit::Changed { .. }
        ));
    }

    #[test]
    fn an_entry_whose_key_is_not_on_the_first_line_of_it_is_read_all_the_same() {
        let by_hand = "suppressions:\n  - reason: ours\n    finding_key: \"user|group|docker\"\n";

        assert_eq!(named(by_hand), vec!["user|group|docker".to_string()]);
    }

    #[test]
    fn a_quotation_mark_in_a_key_does_not_end_the_string_early_and_comes_back_whole() {
        let odd = "user|sshkey|deploy|a\"b";

        let after = changed(add(SHIPPED, &[entry(odd)]));

        assert!(after.contains("\"user|sshkey|deploy|a\\\"b\""), "{after}");
        assert_eq!(named(&after), vec![odd.to_string()]);
    }
}
