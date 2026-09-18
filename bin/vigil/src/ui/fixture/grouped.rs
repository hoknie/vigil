use vigil_model::Snapshot;
use vigil_view::{Cell, Column, Notice, Pane, Room, RowKey, Section, Showing, Width};

pub const SECTION: &str = "containers";

pub const HOST: &str = "host";

pub const DOCKER: &str = "docker";

pub const PODMAN: &str = "podman";

pub const NOT_INSTALLED: &str = "podman is not installed on this host";

pub struct Engines {
    grouped: bool,
}

pub fn section() -> Box<dyn Section> {
    Box::new(Engines { grouped: true })
}

pub fn flat() -> Box<dyn Section> {
    Box::new(Engines { grouped: false })
}

impl Section for Engines {
    fn name(&self) -> &'static str {
        SECTION
    }

    fn title(&self) -> &'static str {
        "What is running in containers"
    }

    fn holds(&self) -> &'static str {
        "what runs in containers"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        let listed = |named, group, here| {
            Box::new(Listed {
                named,
                group: self.grouped.then_some(group),
                here,
            }) as Box<dyn Pane>
        };

        vec![
            listed("containers", HOST, true),
            listed("images", DOCKER, true),
            listed("volumes", DOCKER, true),
            listed("networks", DOCKER, true),
            listed("compose", DOCKER, true),
            listed("pods", PODMAN, false),
        ]
    }

    fn groups(&self) -> Vec<&'static str> {
        match self.grouped {
            true => vec![HOST, DOCKER, PODMAN],
            false => Vec::new(),
        }
    }
}

struct Listed {
    named: &'static str,
    group: Option<&'static str>,
    here: bool,
}

impl Pane for Listed {
    fn name(&self) -> &str {
        self.named
    }

    fn belongs_to(&self) -> Option<&'static str> {
        self.group
    }

    fn caption(&self) -> &str {
        self.named
    }

    fn about(&self) -> &str {
        "what the engine holds on this host"
    }

    fn reads(&self) -> &str {
        "containers"
    }

    fn shown(&self, _reading: &Snapshot) -> bool {
        self.here
    }

    fn columns(&self, _room: Room) -> Vec<Column> {
        vec![
            Column::new("NAME", Width::Fixed(14)),
            Column::new("OF", Width::Least(10)),
        ]
    }

    fn rows(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Vec<RowKey> {
        reading.items.keys().map(RowKey::of).collect()
    }

    fn cells(&self, _reading: &Snapshot, row: &RowKey, _room: Room) -> Vec<Cell> {
        vec![Cell::plain(self.named), Cell::plain(row.key.clone())]
    }

    fn detail(&self, _reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<vigil_view::Piece> {
        vec![vigil_view::Piece::field("key", row.key.clone())]
    }

    fn tally(&self, _reading: &Snapshot, _showing: &Showing<'_>, shown: usize) -> String {
        format!("{shown} of {}", self.named)
    }

    fn empty(&self, _showing: &Showing<'_>) -> Notice {
        Notice::plain(format!("nothing in {}", self.named))
    }

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        Some(Notice::plain(NOT_INSTALLED))
    }
}
