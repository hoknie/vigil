use vigil_model::Snapshot;
use vigil_view::{
    Cell, Column, Notice, Offers, Pane, Piece, Room, RowKey, Showing, Sorting, Width,
};

use super::super::detail;
use super::super::fields;
use super::super::graph;
use crate::types::Kind;

const ROOM_FOR_THE_COUNT: u16 = 100;

const PACKETS: u16 = 14;

const NOT_COUNTED: &str = "—";

const SORTED_BY: &[&str] = &["LINK", "ADDRESS", "PACKETS"];

pub(in crate::views) struct TheInterfaces;

impl TheInterfaces {
    fn listed(reading: &Snapshot, showing: &Showing<'_>) -> Vec<String> {
        let mut keys: Vec<String> = reading
            .items
            .iter()
            .filter(|(key, _)| Kind::of(key) == Some(Kind::Interface))
            .filter(|(key, item)| showing.matches(key, item))
            .map(|(key, _)| key.clone())
            .collect();

        TheInterfaces::sorted(reading, &mut keys, showing.sorting);
        keys
    }

    fn sorted(reading: &Snapshot, keys: &mut [String], sorting: Sorting) {
        if sorting.as_read() {
            return;
        }
        keys.sort_by(|left, right| {
            let ordering = TheInterfaces::sorted_on(reading, left, sorting.by)
                .cmp(&TheInterfaces::sorted_on(reading, right, sorting.by));
            match sorting.descending {
                true => ordering.reverse(),
                false => ordering,
            }
        });
    }

    fn sorted_on(reading: &Snapshot, key: &str, by: usize) -> String {
        let Some(item) = reading.items.get(key) else {
            return String::new();
        };
        match by {
            1 => fields::what(key, item),
            2 => fields::shown_addresses(item),
            3 => format!("{:0>20}", fields::packets(item).unwrap_or_default()),
            _ => String::new(),
        }
    }
}

impl Pane for TheInterfaces {
    fn name(&self) -> &str {
        "the interfaces"
    }

    fn caption(&self) -> &str {
        "WHERE PACKETS ARRIVE"
    }

    fn about(&self) -> &str {
        "every interface of this host, the address it answers on, and the path a packet takes \
         through the rules once it arrives there"
    }

    fn reads(&self) -> &str {
        "firewall"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let mut columns = vec![
            Column::new("LINK", Width::Fixed(12)),
            Column::new("ADDRESS", Width::Share(1)),
            Column::new("OUT", Width::Fixed(4)),
        ];
        if room.holds(ROOM_FOR_THE_COUNT) {
            columns.push(Column::new("PACKETS", Width::Fixed(PACKETS)));
        }
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        TheInterfaces::listed(reading, showing)
            .into_iter()
            .map(RowKey::of)
            .collect()
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };

        let mut cells = vec![
            Cell::plain(fields::what(&row.key, item)),
            Cell::plain(fields::shown_addresses(item)),
            Cell::plain(match fields::the_way_out(item) {
                true => "yes",
                false => "",
            }),
        ];
        if room.holds(ROOM_FOR_THE_COUNT) {
            cells.push(Cell::plain(match fields::packets(item) {
                Some(packets) => packets.to_string(),
                None => NOT_COUNTED.to_string(),
            }));
        }
        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        match reading.items.get(&row.key) {
            Some(item) => detail::of(&row.key, item),
            None => Vec::new(),
        }
    }

    fn graph(&self, reading: &Snapshot, row: &RowKey) -> Vec<Piece> {
        match reading.items.get(&row.key) {
            Some(item) => graph::drawn(reading, &row.key, item),
            None => Vec::new(),
        }
    }

    fn counted(&self, reading: &Snapshot, row: &RowKey) -> Option<u64> {
        fields::packets(reading.items.get(&row.key)?)
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        let counting = TheInterfaces::listed(reading, &Showing::default())
            .iter()
            .filter_map(|key| reading.items.get(key))
            .any(fields::counted);

        let mut parts = vec![
            format!("{shown} interface(s) of this host"),
            match counting {
                true => "counting what goes through".to_string(),
                false => "not counting what goes through".to_string(),
            },
        ];
        if let Some(reason) = showing.note {
            parts.push(format!("incomplete: {reason}"));
        }

        parts.join(" · ")
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        match showing.holding_back() {
            true => Notice::plain(format!("No interface matches {:?}.", showing.search)),
            false => Notice::plain("This reading lists no interface of this host.").saying(
                "The interfaces are read from /proc/net, which a container without its own \
                 network namespace shows and one with a hidden /proc does not.",
            ),
        }
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        SORTED_BY.to_vec()
    }

    fn nothing_was_read(&self) -> &'static str {
        "No interface is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().graphed(true)
    }
}
