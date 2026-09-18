use vigil_model::Snapshot;
use vigil_view::{
    Cell, Column, Counts, Facet, Index, Notice, Offers, Pane, Piece, Room, RowKey, Rows, Showing,
};

use super::cells::{Row, value};
use super::columns::{columns, drawn, sorted_by};
use super::detail::detail;
use super::notices::{empty, why};
use super::rows::{facets_of, index, rows};
use super::tally::{counts, tallied};
use crate::helpers::parts_of;
use crate::parsers::SOURCE;
use crate::types::{Engine, List, Standing};

pub struct Of {
    engine: Engine,
    list: List,
}

impl Of {
    pub fn new(engine: Engine, list: List) -> Of {
        Of { engine, list }
    }

    fn mine(&self, key: &str) -> bool {
        parts_of(key).is_some_and(|(engine, subject, _)| {
            engine == self.engine && subject == self.list.subject()
        })
    }
}

impl Pane for Of {
    fn name(&self) -> &str {
        self.list.name()
    }

    fn belongs_to(&self) -> Option<&'static str> {
        Some(self.engine.name())
    }

    fn caption(&self) -> &str {
        caption(self.engine, self.list)
    }

    fn detail_caption(&self) -> &'static str {
        self.list.detail()
    }

    fn about(&self) -> &str {
        self.list.about()
    }

    fn shown(&self, reading: &Snapshot) -> bool {
        Standing::in_reading(reading, self.engine).answers()
    }

    fn reads(&self) -> &str {
        SOURCE
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        columns(self.list, room)
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        rows(reading, self.engine, self.list, showing)
    }

    fn index(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        Some(index(reading, self.engine, self.list))
    }

    fn counts(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Counts> {
        Some(counts(reading, self.engine, self.list))
    }

    fn tally_listed(
        &self,
        reading: &Snapshot,
        showing: &Showing<'_>,
        rows: &Rows<'_>,
        counts: &Counts,
    ) -> String {
        tallied(reading, self.engine, self.list, showing, rows.len(), counts)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        let row = Row {
            engine: self.engine,
            list: self.list,
            key: &row.key,
            item,
            reading,
        };
        drawn(self.list, room)
            .into_iter()
            .map(|column| Cell::plain(value(&row, column)))
            .collect()
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        match reading.items.get(&row.key) {
            Some(item) => detail(self.engine, self.list, &row.key, item),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tallied(
            reading,
            self.engine,
            self.list,
            showing,
            shown,
            &counts(reading, self.engine, self.list),
        )
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        empty(self.engine, self.list, showing)
    }

    fn why_nothing_is_listed(&self, reading: &Snapshot, showing: &Showing<'_>) -> Option<Notice> {
        Some(why(reading, self.engine, self.list, showing))
    }

    fn nothing_was_read(&self) -> &'static str {
        "Nothing any container engine holds is listed here: nothing was read."
    }

    fn row_for(&self, reading: &Snapshot, named: &str) -> Option<String> {
        match self.mine(named) && reading.items.contains_key(named) {
            true => Some(named.to_string()),
            false => None,
        }
    }

    fn holds(&self, reading: &Snapshot, key: &str) -> bool {
        self.row_for(reading, key).is_some()
    }

    fn facets(&self, reading: &Snapshot, row: &RowKey) -> Vec<Facet> {
        match (self.list.faceted(), reading.items.get(&row.key)) {
            (true, Some(item)) => facets_of(item),
            _ => Vec::new(),
        }
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        sorted_by(self.list)
    }

    fn offers(&self) -> Offers {
        Offers::default().suppressed(true)
    }
}

fn caption(engine: Engine, list: List) -> &'static str {
    match (engine, list) {
        (Engine::Docker, List::Containers) => "CONTAINERS OF DOCKER",
        (Engine::Docker, List::Images) => "IMAGES OF DOCKER",
        (Engine::Docker, List::Volumes) => "VOLUMES OF DOCKER",
        (Engine::Docker, List::Networks) => "NETWORKS OF DOCKER",
        (Engine::Docker, List::Compose) => "COMPOSE PROJECTS OF DOCKER",
        (Engine::Docker, List::Pods) => "PODS OF DOCKER",
        (Engine::Docker, List::Secrets) => "SECRETS OF DOCKER",
        (Engine::Docker, List::Registries) => "REGISTRIES OF DOCKER",
        (Engine::Podman, List::Containers) => "CONTAINERS OF PODMAN",
        (Engine::Podman, List::Images) => "IMAGES OF PODMAN",
        (Engine::Podman, List::Volumes) => "VOLUMES OF PODMAN",
        (Engine::Podman, List::Networks) => "NETWORKS OF PODMAN",
        (Engine::Podman, List::Compose) => "COMPOSE PROJECTS OF PODMAN",
        (Engine::Podman, List::Pods) => "PODS OF PODMAN",
        (Engine::Podman, List::Secrets) => "SECRETS OF PODMAN",
        (Engine::Podman, List::Registries) => "REGISTRIES OF PODMAN",
    }
}
