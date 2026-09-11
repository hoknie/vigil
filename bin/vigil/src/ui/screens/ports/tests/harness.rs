use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde_json::json;
use vigil_model::Snapshot;

use crate::ui::helpers::words::text;
use crate::ui::screens::ports::{Arrangement, Showing, render};
use crate::ui::{Arrows, Protocols, Reading, Search, View, fixture};

#[derive(Default)]
pub(super) struct Given {
    pub(super) protocols: Protocols,
    search: Search,
    arrangement: Arrangement,
}

impl Given {
    pub(super) fn showing(&self, cursor: usize) -> Showing<'_> {
        Showing {
            arrangement: self.arrangement,
            protocols: &self.protocols,
            search: &self.search,
            cursor,
            arrows: Arrows::List,
        }
    }
}

pub(super) fn drawn(view: &View, cursor: usize, width: u16) -> String {
    drawn_with(view, &Given::default(), cursor, width)
}

pub(super) fn drawn_with(view: &View, given: &Given, cursor: usize, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 24));
    render(
        view,
        fixture::look(),
        &given.showing(cursor),
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

pub(super) fn showing(protocols: Protocols, arrangement: Arrangement) -> Given {
    Given {
        protocols,
        arrangement,
        search: Search::default(),
    }
}

pub(super) fn looking_for(wanted: &str) -> Given {
    let mut given = Given::default();
    given.search.start();
    for character in wanted.chars() {
        given.search.type_character(character);
    }
    given.search.accept();
    given
}

pub(super) fn busy() -> View {
    let mut view = fixture::view();
    view.readings.put(
        "ports",
        Reading::Taken(
            Snapshot::new("ports", "2026-09-09T09:00:00.000Z")
                .with(
                    "tcp|0.0.0.0:80",
                    json!({
                        "protocol": "tcp", "address": "0.0.0.0", "port": 80, "uid": 0,
                        "user": "root",
                        "process": {"exe": "/usr/sbin/nginx", "exe_deleted": false,
                                    "cmdline": "nginx -g daemon off;", "cmdline_redacted": false},
                        "owner_resolved": true,
                    }),
                )
                .with(
                    "tcp6|[::]:80",
                    json!({
                        "protocol": "tcp6", "address": "::", "port": 80, "uid": 0,
                        "user": "root",
                        "process": {"exe": "/usr/sbin/nginx", "exe_deleted": false,
                                    "cmdline": "nginx -g daemon off;", "cmdline_redacted": false},
                        "owner_resolved": true,
                    }),
                )
                .with(
                    "udp|0.0.0.0:53",
                    json!({
                        "protocol": "udp", "address": "0.0.0.0", "port": 53, "uid": 0,
                        "user": "root",
                        "process": {"exe": "/usr/sbin/dnsmasq", "exe_deleted": false,
                                    "cmdline": "dnsmasq -k", "cmdline_redacted": false},
                        "owner_resolved": true,
                    }),
                )
                .with(
                    "tcp|127.0.0.11:41857",
                    json!({
                        "protocol": "tcp", "address": "127.0.0.11", "port": 41857, "uid": 0,
                        "user": null, "process": null, "owner_resolved": false,
                    }),
                ),
        ),
    );
    view
}
