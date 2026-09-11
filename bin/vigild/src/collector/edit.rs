const COLLECTORS: &str = "collectors:";

const SCHEDULE: &str = "schedule:";

const ITEM: &str = "  - ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Changed(String),
    AlreadySo,
    NotOurs(String),
}

pub fn watching(text: &str, name: &str) -> Option<bool> {
    match block(text, COLLECTORS)? {
        Block::Missing => None,
        Block::Listed(items) => Some(items.iter().any(|listed| listed == name)),
    }
}

pub fn add(text: &str, name: &str, every_seconds: u32) -> Edit {
    let listed = match block(text, COLLECTORS) {
        None => return Edit::NotOurs(shape_we_do_not_edit(COLLECTORS)),
        Some(Block::Missing) => {
            return Edit::AlreadySo;
        }
        Some(Block::Listed(items)) => items,
    };

    if listed.iter().any(|item| item == name) {
        return match scheduled(text, name) {
            Some(true) => Edit::AlreadySo,
            Some(false) => with_period(text, name, every_seconds),
            None => Edit::NotOurs(shape_we_do_not_edit(SCHEDULE)),
        };
    }

    let with_the_collector = insert(text, COLLECTORS, &format!("{ITEM}{name}"));
    match scheduled(&with_the_collector, name) {
        Some(true) => Edit::Changed(with_the_collector),
        Some(false) => match with_period(&with_the_collector, name, every_seconds) {
            Edit::Changed(text) => Edit::Changed(text),
            Edit::AlreadySo => Edit::Changed(with_the_collector),
            refusal => refusal,
        },
        None => Edit::NotOurs(shape_we_do_not_edit(SCHEDULE)),
    }
}

pub fn remove(text: &str, name: &str, every: &[(&str, u32)]) -> Edit {
    let listed = match block(text, COLLECTORS) {
        None => return Edit::NotOurs(shape_we_do_not_edit(COLLECTORS)),
        Some(Block::Missing) => {
            let named: Vec<String> = every
                .iter()
                .filter(|(known, _)| *known != name)
                .map(|(known, _)| format!("{ITEM}{known}"))
                .collect();
            let mut text = text.trim_end().to_string();
            text.push_str(&format!("\n\n{COLLECTORS}\n{}\n", named.join("\n")));
            return Edit::Changed(without_period(&text, name));
        }
        Some(Block::Listed(items)) => items,
    };

    if !listed.iter().any(|item| item == name) {
        return match scheduled(text, name) {
            Some(true) => Edit::Changed(without_period(text, name)),
            _ => Edit::AlreadySo,
        };
    }

    let text = drop_line(text, &format!("{ITEM}{name}"));
    Edit::Changed(without_period(&text, name))
}

fn with_period(text: &str, name: &str, every_seconds: u32) -> Edit {
    match block(text, SCHEDULE) {
        None => Edit::NotOurs(shape_we_do_not_edit(SCHEDULE)),
        Some(Block::Missing) => {
            let mut text = text.trim_end().to_string();
            text.push_str(&format!("\n\n{SCHEDULE}\n  {name}: {every_seconds}\n"));
            Edit::Changed(text)
        }
        Some(Block::Listed(_)) => Edit::Changed(insert(
            text,
            SCHEDULE,
            &format!("  {name}: {every_seconds}"),
        )),
    }
}

fn without_period(text: &str, name: &str) -> String {
    let prefix = format!("  {name}:");
    text.lines()
        .filter(|line| !line.starts_with(&prefix))
        .map(|line| format!("{line}\n"))
        .collect()
}

fn scheduled(text: &str, name: &str) -> Option<bool> {
    match block(text, SCHEDULE)? {
        Block::Missing => Some(false),
        Block::Listed(_) => Some(
            text.lines()
                .any(|line| line.starts_with(&format!("  {name}:"))),
        ),
    }
}

enum Block {
    Missing,
    Listed(Vec<String>),
}

fn block(text: &str, heading: &str) -> Option<Block> {
    let mut lines = text.lines();
    let Some(start) = lines.position(|line| line.trim_end() == heading) else {
        return match text.lines().any(|line| line.starts_with(heading)) {
            true => None,
            false => Some(Block::Missing),
        };
    };

    let mut items = Vec::new();
    for line in text.lines().skip(start + 1) {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        let held = line.trim_start().trim_start_matches("- ").trim();
        items.push(held.split(':').next().unwrap_or(held).trim().to_string());
    }
    Some(Block::Listed(items))
}

fn insert(text: &str, heading: &str, line: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    let mut placed = false;

    for source in text.lines() {
        if inside && !placed && (source.trim().is_empty() || !source.starts_with(' ')) {
            out.push_str(line);
            out.push('\n');
            placed = true;
            inside = false;
        }
        out.push_str(source);
        out.push('\n');
        if source.trim_end() == heading {
            inside = true;
        }
    }
    if inside && !placed {
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn drop_line(text: &str, wanted: &str) -> String {
    text.lines()
        .filter(|line| line.trim_end() != wanted)
        .map(|line| format!("{line}\n"))
        .collect()
}

fn shape_we_do_not_edit(heading: &str) -> String {
    format!(
        "the {heading} in this file is not written as a block of lines, and this command edits \
         no shape it did not write. Add the line by hand, or write the file again with `vigild \
         configure --force`"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = "\
state_dir: /var/lib/vigil
socket_path: /run/vigil/vigil.sock
retention_days: 90

collectors:
  - ports
  - users

schedule:
  ports: 30
  users: 300

suppressions: []

reporters: []
";

    fn changed(edit: Edit) -> String {
        match edit {
            Edit::Changed(text) => text,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_collector_added_leaves_every_other_byte_of_the_file_where_it_was() {
        let after = changed(add(SHIPPED, "firewall", 60));

        assert!(after.contains("  - firewall\n"), "{after}");
        assert!(after.contains("  firewall: 60\n"), "{after}");
        for kept in [
            "state_dir: /var/lib/vigil",
            "socket_path: /run/vigil/vigil.sock",
            "retention_days: 90",
            "  - ports",
            "  - users",
            "  ports: 30",
            "  users: 300",
            "suppressions: []",
            "reporters: []",
        ] {
            assert!(after.contains(kept), "{kept} was lost:\n{after}");
        }
        assert_eq!(
            after.lines().count(),
            SHIPPED.lines().count() + 2,
            "exactly two lines were added:\n{after}"
        );
    }

    #[test]
    fn adding_a_collector_that_is_already_watched_changes_nothing_at_all() {
        assert_eq!(add(SHIPPED, "ports", 30), Edit::AlreadySo);
        let once = changed(add(SHIPPED, "firewall", 60));
        assert_eq!(add(&once, "firewall", 60), Edit::AlreadySo);
    }

    #[test]
    fn a_disabled_collector_leaves_no_period_behind_because_the_daemon_refuses_one() {
        let with_it = changed(add(SHIPPED, "firewall", 60));

        let after = changed(remove(&with_it, "firewall", &[]));

        assert!(!after.contains("firewall"), "{after}");
        assert_eq!(
            after, SHIPPED,
            "what enable put in, disable takes out, and the file is the one it started as"
        );
    }

    #[test]
    fn removing_one_that_is_not_there_changes_nothing() {
        assert_eq!(remove(SHIPPED, "firewall", &[]), Edit::AlreadySo);
    }

    #[test]
    fn a_file_that_names_no_collectors_means_all_of_them_and_saying_so_is_not_an_edit() {
        let silent = "state_dir: /var/lib/vigil\nreporters: []\n";

        assert_eq!(add(silent, "firewall", 60), Edit::AlreadySo);
    }

    #[test]
    fn taking_one_out_of_a_file_that_named_none_writes_the_rest_by_name() {
        let silent = "state_dir: /var/lib/vigil\nreporters: []\n";
        let known = [("ports", 30u32), ("firewall", 60)];

        let after = changed(remove(silent, "firewall", &known));

        assert!(after.contains("  - ports"), "{after}");
        assert!(!after.contains("firewall"), "{after}");
        assert!(after.contains("state_dir: /var/lib/vigil"), "{after}");
    }

    #[test]
    fn a_shape_this_command_did_not_write_is_left_alone_and_named() {
        let inline = "collectors: [ports, users]\nschedule: {ports: 30}\n";

        match add(inline, "firewall", 60) {
            Edit::NotOurs(said) => assert!(said.contains("by hand"), "{said}"),
            other => panic!("a file written another way must not be rewritten: {other:?}"),
        }
    }

    #[test]
    fn a_file_with_no_schedule_block_gets_one_rather_than_a_period_with_nowhere_to_go() {
        let no_schedule = "collectors:\n  - ports\n";

        let after = changed(add(no_schedule, "firewall", 60));

        assert!(after.contains("schedule:\n"), "{after}");
        assert!(after.contains("  firewall: 60"), "{after}");
    }

    #[test]
    fn what_the_file_says_now_is_read_before_anything_is_written() {
        assert_eq!(watching(SHIPPED, "ports"), Some(true));
        assert_eq!(watching(SHIPPED, "firewall"), Some(false));
        assert_eq!(
            watching("state_dir: /x\n", "firewall"),
            None,
            "a file that names no collectors is watching every one of them, which is not the \
             same answer as naming this one"
        );
    }
}
