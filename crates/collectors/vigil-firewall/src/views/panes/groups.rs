use vigil_model::Snapshot;
use vigil_view::{Cell, Column, Notice, Offers, Pane, Piece, Room, RowKey, Showing, Width};

use super::super::detail;
use super::super::fields;
use crate::types::{Kind, Zone};

pub(in crate::views) const ZONE: &str = "fw-zone|";

const ROOM_FOR_WHERE: u16 = 100;

const NONE_OF_ITS_OWN: &str = "—";

pub(in crate::views) struct TheGroups;

impl TheGroups {
    fn tables(reading: &Snapshot, showing: &Showing<'_>) -> Vec<String> {
        reading
            .items
            .iter()
            .filter(|(key, _)| Kind::of(key) == Some(Kind::Table))
            .filter(|(key, item)| showing.matches(key, item))
            .map(|(key, _)| key.clone())
            .collect()
    }

    fn zones(reading: &Snapshot, showing: &Showing<'_>) -> Vec<Zone> {
        Zone::of(reading)
            .into_iter()
            .filter(|zone| {
                showing.search.is_empty()
                    || format!("{} {}", zone.name, zone.shown_on())
                        .to_lowercase()
                        .contains(&showing.search.to_lowercase())
            })
            .collect()
    }

    fn zone_named(reading: &Snapshot, key: &str) -> Option<Zone> {
        let name = key.strip_prefix(ZONE)?;

        Zone::of(reading).into_iter().find(|zone| zone.name == name)
    }
}

impl Pane for TheGroups {
    fn name(&self) -> &str {
        "the groups"
    }

    fn caption(&self) -> &str {
        "GROUPS AND ZONES"
    }

    fn about(&self) -> &str {
        "the tables this host's rules are grouped into, and the zones firewalld built out of \
         chains where it is the one holding them"
    }

    fn reads(&self) -> &str {
        "firewall"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let mut columns = vec![
            Column::new("KIND", Width::Fixed(6)),
            Column::new("NAME", Width::Share(1)),
            Column::new("CHAINS", Width::Fixed(7)),
            Column::new("RULES", Width::Fixed(6)),
        ];
        if room.holds(ROOM_FOR_WHERE) {
            columns.push(Column::new("REACHED FROM", Width::Share(1)));
        }
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows: Vec<RowKey> = TheGroups::tables(reading, showing)
            .into_iter()
            .map(RowKey::of)
            .collect();

        rows.extend(
            TheGroups::zones(reading, showing)
                .into_iter()
                .map(|zone| RowKey::of(format!("{ZONE}{}", zone.name)).of_its_own()),
        );
        rows
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let mut cells = match TheGroups::zone_named(reading, &row.key) {
            Some(zone) => vec![
                Cell::plain("zone"),
                Cell::plain(zone.name.clone()),
                Cell::plain(zone.through.len().to_string()),
                Cell::plain(NONE_OF_ITS_OWN),
            ],
            None => {
                let Some(item) = reading.items.get(&row.key) else {
                    return Vec::new();
                };
                vec![
                    Cell::plain("table"),
                    Cell::plain(fields::what(&row.key, item)),
                    Cell::plain(fields::chains(item)),
                    Cell::plain(fields::rules(item)),
                ]
            }
        };

        if room.holds(ROOM_FOR_WHERE) {
            cells.push(Cell::plain(
                match TheGroups::zone_named(reading, &row.key) {
                    Some(zone) => zone.shown_on(),
                    None => NONE_OF_ITS_OWN.to_string(),
                },
            ));
        }
        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        if let Some(zone) = TheGroups::zone_named(reading, &row.key) {
            return detail::zone(&zone);
        }
        match reading.items.get(&row.key) {
            Some(item) => detail::of(&row.key, item),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        let zones = TheGroups::zones(reading, &Showing::default()).len();
        let mut parts = vec![
            format!("{shown} group(s) shown"),
            match zones {
                0 => "this backend has no zones".to_string(),
                one => format!("{one} zone(s) firewalld built"),
            },
        ];
        if let Some(reason) = showing.note {
            parts.push(format!("incomplete: {reason}"));
        }

        parts.join(" · ")
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        match showing.holding_back() {
            true => Notice::plain(format!(
                "No table and no zone matches {:?}.",
                showing.search
            )),
            false => Notice::plain("This ruleset is grouped into no table at all.").saying(
                "A rule lives in a chain and a chain lives in a table, so a host with no \
                 table has no rule either — which is not the same as a host whose rules this \
                 build could not read.",
            ),
        }
    }

    fn nothing_was_read(&self) -> &'static str {
        "No table and no zone is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().sorted(false)
    }
}
