use std::collections::BTreeMap;

use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{
    Cell, Column, Index, Notice, Pane, Piece, Room, RowKey, Showing, Width, basename, haystack,
    time_of_day,
};

use crate::parsers::SWITCHED_OFF;
use crate::types::PeerView;

#[derive(Clone, Copy)]
pub(super) enum Direction {
    Out,
    In,
}

pub(super) struct Pairs(pub(super) Direction);

impl Direction {
    fn prefix(self) -> &'static str {
        match self {
            Direction::Out => "outbound|",
            Direction::In => "inbound|",
        }
    }

    fn beyond(self) -> &'static str {
        match self {
            Direction::Out => "outbound|beyond",
            Direction::In => "inbound|beyond",
        }
    }

    fn counted(self) -> &'static str {
        match self {
            Direction::Out => "program-to-peer pair(s)",
            Direction::In => "peer-to-program pair(s)",
        }
    }
}

impl Pairs {
    fn listed<'a>(&self, reading: &'a Snapshot) -> impl Iterator<Item = (&'a String, &'a Value)> {
        let direction = self.0;
        reading.items.iter().filter(move |(key, _)| {
            key.starts_with(direction.prefix()) && key.as_str() != direction.beyond()
        })
    }

    fn values(&self, reading: &Snapshot, key: &str, item: &Value) -> Vec<String> {
        let pair = PeerView::new(key, item);
        let program = shown_program(reading, self, &pair);
        match self.0 {
            Direction::Out => vec![
                program,
                format!("{}:{}", pair.remote_address(), pair.remote_port()),
                pair.protocol().to_string(),
                pair.user(),
            ],
            Direction::In => vec![
                program,
                pair.local_port().to_string(),
                pair.remote_address().to_string(),
                pair.protocol().to_string(),
                pair.user(),
            ],
        }
    }
}

impl Pane for Pairs {
    fn name(&self) -> &str {
        match self.0 {
            Direction::Out => "outbound",
            Direction::In => "inbound",
        }
    }

    fn caption(&self) -> &str {
        match self.0 {
            Direction::Out => "TALKING OUT",
            Direction::In => "TALKING IN",
        }
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED PAIR"
    }

    fn about(&self) -> &str {
        match self.0 {
            Direction::Out => {
                "one row per program and the peer it talks to, however many connections it holds"
            }
            Direction::In => {
                "one row per peer and the port of this host it talks to, however many connections it holds"
            }
        }
    }

    fn reads(&self) -> &str {
        "network"
    }

    fn columns(&self, _room: Room) -> Vec<Column> {
        match self.0 {
            Direction::Out => vec![
                Column::new("PROGRAM", Width::Share(1)),
                Column::new("PEER", Width::Least(22)),
                Column::new("PROTO", Width::Fixed(5)),
                Column::new("USER", Width::Fixed(10)),
            ],
            Direction::In => vec![
                Column::new("PROGRAM", Width::Share(1)),
                Column::new("PORT", Width::Fixed(6)),
                Column::new("PEER", Width::Least(16)),
                Column::new("PROTO", Width::Fixed(5)),
                Column::new("USER", Width::Fixed(10)),
            ],
        }
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows: Vec<(Vec<String>, String)> = self
            .listed(reading)
            .filter(|(key, item)| showing.matches(key, item))
            .map(|(key, item)| (self.values(reading, key, item), key.clone()))
            .collect();

        if !showing.sorting.as_read() {
            let by = showing.sorting.by.saturating_sub(1);
            rows.sort_by(|left, right| {
                let ordering = left.0.get(by).cmp(&right.0.get(by));
                match showing.sorting.descending {
                    true => ordering.reverse(),
                    false => ordering,
                }
            });
        }
        rows.into_iter().map(|(_, key)| RowKey::of(key)).collect()
    }

    fn index(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        let mut index = Index::new(self.sorted_by().len());
        for (key, item) in self.listed(reading) {
            index.push(
                RowKey::of(key.clone()),
                &haystack(key, item),
                0,
                self.values(reading, key, item),
            );
        }
        Some(index)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, _room: Room) -> Vec<Cell> {
        match reading.items.get(&row.key) {
            Some(item) => self
                .values(reading, &row.key, item)
                .into_iter()
                .map(Cell::plain)
                .collect(),
            None => Vec::new(),
        }
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        let pair = PeerView::new(&row.key, item);
        let mut said = vec![
            Piece::field("program", pair.program().unwrap_or("owner not resolved")),
            Piece::field("user", pair.user()),
            Piece::field("protocol", pair.protocol()),
        ];
        match self.0 {
            Direction::Out => said.push(Piece::field(
                "peer",
                format!("{}:{}", pair.remote_address(), pair.remote_port()),
            )),
            Direction::In => {
                said.push(Piece::field(
                    "port of this host",
                    pair.local_port().to_string(),
                ));
                said.push(Piece::field("peer", pair.remote_address()));
            }
        }
        if pair.program_deleted() {
            said.push(Piece::warning(
                "The program was deleted from disk while it was still running.",
            ));
        }
        said.push(Piece::text(
            "A connection opened and closed between two readings is not shown here, and \
             however many connections the pair holds it is one row.",
        ));
        said.push(Piece::Blank);
        said
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        let whole = self.listed(reading).count();
        let read_at = time_of_day(&reading.taken_at);
        let mut said = match showing.narrowed() {
            false => format!("{whole} {}, read at {read_at}", self.0.counted()),
            true => format!("{shown} of {whole} {}, read at {read_at}", self.0.counted()),
        };
        if let Some(beyond) = reading
            .items
            .get(self.0.beyond())
            .and_then(|row| row["count"].as_u64())
        {
            said.push_str(&format!(" · {beyond} more not recorded"));
        }
        said
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        match (showing.narrowed(), self.0) {
            (true, _) => Notice::plain("No pair is left once the list is narrowed.")
                .saying("Press / to change the search, Esc to drop it."),
            (false, Direction::Out) => {
                Notice::plain("This host is not talking to anyone outside it.")
            }
            (false, Direction::In) => Notice::plain("Nobody outside this host is talking to it."),
        }
    }

    fn why_nothing_is_listed(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Notice> {
        reading.items.contains_key(SWITCHED_OFF).then(|| {
            Notice::plain(
                "Connections are not read here: connections is false in the network block.",
            )
        })
    }

    fn nothing_was_read(&self) -> &'static str {
        "No pair is listed here: nothing was read."
    }
}

fn shown_program(reading: &Snapshot, pairs: &Pairs, pair: &PeerView<'_>) -> String {
    let Some(path) = pair.program() else {
        return "owner not resolved".to_string();
    };
    let name = basename(path);
    let mut paths: BTreeMap<&str, ()> = BTreeMap::new();
    for (_, item) in pairs.listed(reading) {
        if let Some(other) = item["program"].as_str()
            && basename(other) == name
        {
            paths.insert(other, ());
        }
    }
    let shown = match paths.len() > 1 {
        true => path.to_string(),
        false => name.to_string(),
    };
    match pair.program_deleted() {
        true => format!("{shown} (deleted)"),
        false => shown,
    }
}
