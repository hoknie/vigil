use serde_json::json;
use vigil_model::Snapshot;

use super::{grouping, suite};
use crate::{Cell, Column, Notice, Pane, Room, RowKey, Section, Showing, Width};

struct Listening {
    short: bool,
    named: &'static str,
    group: Option<&'static str>,
}

impl Pane for Listening {
    fn name(&self) -> &'static str {
        self.named
    }

    fn belongs_to(&self) -> Option<&'static str> {
        self.group
    }

    fn caption(&self) -> &'static str {
        "LISTENING"
    }

    fn about(&self) -> &'static str {
        "every socket this host listens on"
    }

    fn reads(&self) -> &'static str {
        "ports"
    }

    fn columns(&self, _room: Room) -> Vec<Column> {
        vec![
            Column::new("PROTO", Width::Fixed(5)),
            Column::new("ADDRESS", Width::Least(22)),
        ]
    }

    fn rows(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Vec<RowKey> {
        reading.items.keys().map(RowKey::of).collect()
    }

    fn cells(&self, _reading: &Snapshot, row: &RowKey, _room: Room) -> Vec<Cell> {
        match self.short {
            true => vec![Cell::plain(row.key.clone())],
            false => vec![Cell::plain("tcp"), Cell::plain(row.key.clone())],
        }
    }

    fn detail(&self, _reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<crate::Piece> {
        vec![crate::Piece::field("key", row.key.clone())]
    }

    fn tally(&self, reading: &Snapshot, _showing: &Showing<'_>, _shown: usize) -> String {
        format!("{} sockets", reading.items.len())
    }

    fn empty(&self, _showing: &Showing<'_>) -> Notice {
        Notice::plain("nothing is listening")
    }
}

struct Engines {
    named: Vec<&'static str>,
    lists: Vec<(&'static str, Option<&'static str>)>,
}

impl Section for Engines {
    fn name(&self) -> &'static str {
        "containers"
    }

    fn title(&self) -> &'static str {
        "What is running in containers"
    }

    fn holds(&self) -> &'static str {
        "what runs in containers"
    }

    fn panes(&self) -> Vec<Box<dyn Pane>> {
        self.lists
            .iter()
            .map(|(named, group)| {
                Box::new(Listening {
                    short: false,
                    named,
                    group: *group,
                }) as Box<dyn Pane>
            })
            .collect()
    }

    fn groups(&self) -> Vec<&'static str> {
        self.named.clone()
    }
}

fn listening() -> Listening {
    Listening {
        short: false,
        named: "flat",
        group: None,
    }
}

fn reading() -> Snapshot {
    Snapshot::new("ports", "2026-09-12T10:00:00.000Z".to_string()).with(
        "tcp|0.0.0.0:443",
        json!({ "protocol": "tcp", "address": "0.0.0.0", "port": 443 }),
    )
}

#[test]
fn a_pane_that_answers_about_its_own_reading_passes_the_whole_suite() {
    suite::run_all(&listening(), &reading());
}

#[test]
#[should_panic(expected = "cells under")]
fn a_row_shorter_than_the_headers_is_caught_here_and_not_by_a_reader_of_a_shifted_table() {
    suite::a_pane_draws_a_cell_for_every_column_it_declares(
        &Listening {
            short: true,
            ..listening()
        },
        &reading(),
    );
}

#[test]
#[should_panic(expected = "was handed the reading of")]
fn a_pane_given_somebody_elses_reading_is_caught_before_it_draws_it() {
    let elsewhere = Snapshot::new("users", "2026-09-12T10:00:00.000Z".to_string());

    suite::a_pane_reads_the_snapshot_it_says_it_reads(&listening(), &elsewhere);
}

#[test]
fn a_section_whose_lists_carry_the_groups_it_names_passes_the_whole_suite() {
    grouping::run_all_of_the_section(&Engines {
        named: vec!["host", "docker", "podman"],
        lists: vec![
            ("containers", Some("host")),
            ("images", Some("docker")),
            ("volumes", Some("docker")),
            ("pods", Some("podman")),
        ],
    });
}

#[test]
fn a_section_that_names_no_group_and_groups_no_list_passes_it_too() {
    grouping::run_all_of_the_section(&Engines {
        named: Vec::new(),
        lists: vec![("sockets", None), ("by program", None)],
    });
}

#[test]
#[should_panic(expected = "gives a group to")]
fn one_list_left_out_of_the_groups_is_caught_here_and_not_by_a_reader_who_cannot_reach_it() {
    grouping::a_section_gives_every_pane_a_group_or_none_of_them(&Engines {
        named: vec!["host", "docker"],
        lists: vec![("containers", Some("host")), ("images", None)],
    });
}

#[test]
#[should_panic(expected = "and no list of it belongs there")]
fn a_group_no_list_belongs_to_is_caught_before_it_is_drawn_as_a_name_that_opens_nothing() {
    grouping::every_group_the_section_names_is_carried_by_a_pane(&Engines {
        named: vec!["host", "docker", "podman"],
        lists: vec![("containers", Some("host")), ("images", Some("docker"))],
    });
}

#[test]
#[should_panic(expected = "names the groups")]
fn a_first_row_over_lists_that_belong_to_nothing_is_caught_as_a_row_that_never_changes() {
    grouping::a_section_gives_every_pane_a_group_or_none_of_them(&Engines {
        named: vec!["host"],
        lists: vec![("containers", None)],
    });
}
