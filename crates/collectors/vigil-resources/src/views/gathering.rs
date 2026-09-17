use std::sync::Arc;

use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Index, Placed, RowKey, Showing, haystack};

use super::backing::{heading_of, is_a_filesystem};

const NO_FILESYSTEM: usize = usize::MAX;

pub(super) fn wanted<'a>(
    reading: &'a Snapshot,
    showing: &'a Showing<'a>,
) -> impl Iterator<Item = (&'a String, &'a Value)> {
    reading
        .items
        .iter()
        .filter(|(key, item)| is_a_filesystem(key) && showing.matches(key, item))
}

pub(super) fn gathered<'a>(
    reading: &'a Snapshot,
    showing: &'a Showing<'a>,
) -> Vec<(String, Vec<&'a String>)> {
    let mut gathered: Vec<(String, Vec<&'a String>)> = Vec::new();

    for (key, item) in wanted(reading, showing) {
        let heading = heading_of(item);
        match gathered.iter_mut().find(|(named, _)| *named == heading) {
            Some((_, under)) => under.push(key),
            None => gathered.push((heading, vec![key])),
        }
    }
    gathered.sort_by(|left, right| left.0.cmp(&right.0));
    for (_, under) in &mut gathered {
        under.sort();
    }
    gathered
}

pub(super) fn indexed(reading: &Snapshot, showing: &Showing<'_>, columns: usize) -> Index {
    let filesystems: Vec<(&String, &Value, String)> = wanted(reading, showing)
        .map(|(key, item)| (key, item, heading_of(item)))
        .collect();
    let mut headings: Vec<&str> = filesystems
        .iter()
        .map(|(_, _, heading)| heading.as_str())
        .collect();
    headings.sort_unstable();
    headings.dedup();
    let shared: Vec<Arc<str>> = headings.iter().map(|heading| Arc::from(*heading)).collect();

    let mut index = Index::new(columns);
    for (key, item, heading) in &filesystems {
        let at = headings
            .binary_search(&heading.as_str())
            .unwrap_or(headings.len());
        let row = RowKey::of((*key).clone()).under(1);
        let row = match shared.get(at) {
            Some(beneath) => row.beneath(Arc::clone(beneath)),
            None => row,
        };
        index.push(row, &haystack(key, item), 0, Vec::new());
        index.gathered(at);
    }
    index
}

pub(super) fn assembled(showing: &Showing<'_>, index: &Index, ordered: Vec<usize>) -> Vec<Placed> {
    let mut headings: Vec<(usize, usize)> = Vec::new();
    for at in &ordered {
        if let Some(heading) = index.heading_of(*at) {
            if heading >= headings.len() {
                headings.resize(heading + 1, (0, NO_FILESYSTEM));
            }
            let (gathers, first) = &mut headings[heading];
            *gathers += 1;
            if *first == NO_FILESYSTEM {
                *first = *at;
            }
        }
    }

    let mut placed = Vec::new();
    for (gathers, first) in &mut headings {
        if *gathers == 0 {
            continue;
        }
        let opened = index
            .row(*first)
            .gathered_under
            .as_deref()
            .is_some_and(|heading| showing.opened_up(heading));
        placed.push(Placed::Heading {
            first: *first,
            gathers: *gathers,
            opened,
        });
        *first = match opened {
            true => {
                let slot = placed.len();
                placed.resize(slot + *gathers, Placed::Row(NO_FILESYSTEM));
                slot
            }
            false => NO_FILESYSTEM,
        };
    }

    for at in ordered {
        if let Some(heading) = index.heading_of(at)
            && let Some((_, slot)) = headings.get_mut(heading)
            && *slot != NO_FILESYSTEM
        {
            placed[*slot] = Placed::Row(at);
            *slot += 1;
        }
    }
    placed
}
