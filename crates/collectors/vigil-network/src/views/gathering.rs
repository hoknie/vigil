use std::sync::Arc;

use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Index, Placed, RowKey, Showing, haystack};

use super::fields::holder;
use super::flat::wanted;
use super::programs::{HEADING, Programs, UNRESOLVED};

const NO_SOCKET: usize = usize::MAX;

pub(super) fn indexed(reading: &Snapshot, showing: &Showing<'_>, columns: usize) -> Index {
    let names = Programs::whole(reading);
    let sockets: Vec<(&String, &Value, Option<&str>)> = wanted(reading, showing)
        .map(|(key, item)| (key, item, holder(item)))
        .collect();
    let mut programs: Vec<&str> = sockets.iter().filter_map(|(_, _, path)| *path).collect();
    programs.sort_unstable();
    programs.dedup();

    let shared: Vec<(Arc<str>, Arc<str>)> = programs
        .iter()
        .map(|path| {
            (
                Arc::from(format!("{HEADING}{path}")),
                Arc::from(names.name(path)),
            )
        })
        .collect();
    let unresolved: Arc<str> = Arc::from(UNRESOLVED);

    let mut index = Index::new(columns);
    for (key, item, path) in sockets {
        let socket = RowKey::of(key.clone()).under(1);
        let heading = path
            .and_then(|path| programs.binary_search(&path).ok())
            .unwrap_or(programs.len());
        let socket = match shared.get(heading) {
            Some((beneath, name)) => socket.beneath(Arc::clone(beneath)).named(Arc::clone(name)),
            None => socket.beneath(Arc::clone(&unresolved)),
        };
        index.push(socket, &haystack(key, item), 0, Vec::new());
        index.gathered(heading);
    }
    index
}

pub(super) fn assembled(showing: &Showing<'_>, index: &Index, ordered: Vec<usize>) -> Vec<Placed> {
    let mut headings: Vec<(usize, usize)> = Vec::new();
    for at in &ordered {
        if let Some(heading) = index.heading_of(*at) {
            if heading >= headings.len() {
                headings.resize(heading + 1, (0, NO_SOCKET));
            }
            let (gathers, first) = &mut headings[heading];
            *gathers += 1;
            if *first == NO_SOCKET {
                *first = *at;
            }
        }
    }

    let mut placed = Vec::new();
    for (gathers, first) in &mut headings {
        if *gathers == 0 {
            continue;
        }
        let opened = showing.opened_up(
            index
                .row(*first)
                .gathered_under
                .as_deref()
                .unwrap_or(UNRESOLVED),
        );
        placed.push(Placed::Heading {
            first: *first,
            gathers: *gathers,
            opened,
        });
        *first = match opened {
            true => {
                let slot = placed.len();
                placed.resize(slot + *gathers, Placed::Row(NO_SOCKET));
                slot
            }
            false => NO_SOCKET,
        };
    }

    for at in ordered {
        if let Some(heading) = index.heading_of(at)
            && let Some((_, slot)) = headings.get_mut(heading)
            && *slot != NO_SOCKET
        {
            placed[*slot] = Placed::Row(at);
            *slot += 1;
        }
    }
    placed
}
