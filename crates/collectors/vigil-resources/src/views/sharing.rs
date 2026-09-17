use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Counts, Showing, time_of_day};

use super::backing::{heading_of, is_a_filesystem};
use super::fields::{free_step, number, text};

const FILESYSTEMS: &str = "filesystems";

const STORES: &str = "stores";

const TWINS: &str = "filesystems of one size under two headings";

const FULLEST: &str = "the filesystem with the least room left";

const SAME_SIZE: &str = "of the same size under different headings: equal sizes often mean one \
                         device this host does not name";

pub(super) fn counted(reading: &Snapshot) -> Counts {
    let filesystems: Vec<&Value> = reading
        .items
        .iter()
        .filter(|(key, _)| is_a_filesystem(key))
        .map(|(_, item)| item)
        .collect();

    let mut headings: Vec<String> = filesystems.iter().map(|item| heading_of(item)).collect();
    headings.sort();
    headings.dedup();

    Counts::default()
        .counted(FILESYSTEMS, filesystems.len())
        .counted(STORES, headings.len())
        .counted(TWINS, twins(&filesystems))
        .saying(FULLEST, fullest(&filesystems))
}

pub(super) fn tallied(
    reading: &Snapshot,
    showing: &Showing<'_>,
    shown: usize,
    counts: &Counts,
) -> String {
    let read_at = time_of_day(&reading.taken_at);
    let whole = counts.number(FILESYSTEMS);

    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{whole} filesystem(s) on {} store(s), read at {read_at}",
            counts.number(STORES)
        ),
        true => format!(
            "{shown} row(s) of {whole} filesystem(s) on {} store(s), read at {read_at}",
            counts.number(STORES)
        ),
    }];

    if let [mount, step] = counts.words(FULLEST) {
        parts.push(format!("least room on {mount}, {step}% free"));
    }
    match counts.number(TWINS) {
        0 => {}
        counted => parts.push(format!("{counted} {SAME_SIZE}")),
    }
    if showing.elsewhere > 0 {
        parts.push(format!(
            "{} other list(s) narrowed by a search of their own",
            showing.elsewhere
        ));
    }
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }

    parts.join(" \u{b7} ")
}

fn fullest(filesystems: &[&Value]) -> Vec<String> {
    filesystems
        .iter()
        .filter_map(|item| free_step(item).map(|step| (step, text(item, "mount"))))
        .min_by_key(|(step, mount)| (*step, *mount))
        .map(|(step, mount)| vec![mount.to_string(), step.to_string()])
        .unwrap_or_default()
}

fn twins(filesystems: &[&Value]) -> usize {
    let mut sized: Vec<(u64, String)> = filesystems
        .iter()
        .map(|item| (number(item, "total_bytes"), heading_of(item)))
        .filter(|(size, _)| *size > 0)
        .collect();
    sized.sort();

    sized
        .chunk_by(|left, right| left.0 == right.0)
        .filter(|held| held.iter().any(|(_, under)| *under != held[0].1))
        .map(<[(u64, String)]>::len)
        .sum()
}
