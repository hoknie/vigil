use std::collections::BTreeMap;

use vigil_model::Snapshot;
use vigil_view::{Index, RowKey, Showing, haystack};

use super::fields::holder;
use super::flat::wanted;
use super::programs::{HEADING, Programs, UNRESOLVED};

pub(super) fn indexed(reading: &Snapshot, showing: &Showing<'_>, columns: usize) -> Index {
    let names = Programs::whole(reading);
    let mut index = Index::new(columns);
    for (key, item) in wanted(reading, showing) {
        let socket = RowKey::of(key.clone()).under(1);
        let socket = match holder(item) {
            Some(path) => socket
                .beneath(format!("{HEADING}{path}"))
                .named(names.name(path)),
            None => socket.beneath(UNRESOLVED),
        };
        index.push(socket, &haystack(key, item), 0, Vec::new());
    }
    index
}

pub(super) fn assembled(showing: &Showing<'_>, index: &Index, ordered: &[usize]) -> Vec<RowKey> {
    let mut programs: BTreeMap<&str, Vec<&RowKey>> = BTreeMap::new();
    let mut unresolved: Vec<&RowKey> = Vec::new();
    for at in ordered {
        let socket = index.row(*at);
        match socket.gathered_under.as_deref() {
            Some(heading) if heading != UNRESOLVED => {
                programs.entry(heading).or_default().push(socket)
            }
            _ => unresolved.push(socket),
        }
    }

    let mut rows = Vec::new();
    for (heading, sockets) in programs {
        let mut row = RowKey::of(heading)
            .of_its_own()
            .gathering(sockets.len())
            .opened(showing.opened_up(heading));
        row.named = sockets.first().and_then(|socket| socket.named.clone());
        folded(&mut rows, row, &sockets);
    }
    if !unresolved.is_empty() {
        let row = RowKey::of(UNRESOLVED)
            .of_its_own()
            .gathering(unresolved.len())
            .opened(showing.opened_up(UNRESOLVED));
        folded(&mut rows, row, &unresolved);
    }
    rows
}

fn folded(rows: &mut Vec<RowKey>, heading: RowKey, sockets: &[&RowKey]) {
    let open = heading.opened;
    rows.push(heading);
    if open {
        rows.extend(sockets.iter().map(|socket| RowKey {
            named: None,
            ..(*socket).clone()
        }));
    }
}
