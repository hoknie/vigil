use serde_json::json;
use vigil_view::{Room, Section, Showing};

use crate::fixture::resources;
use crate::types::Family;
use crate::views::TheHostAndItsFiles;
use crate::views::fields::{free, means, what};

#[test]
fn the_processor_row_sits_between_the_boot_and_memory() {
    let reading = resources();
    let pane = TheHostAndItsFiles.panes().remove(0);

    let kinds: Vec<String> = pane
        .rows(&reading, &Showing::default())
        .iter()
        .map(|row| row.key.split('|').next().unwrap_or("").to_string())
        .take(3)
        .collect();

    assert_eq!(kinds, ["boot", "cpu", "memory"]);
}

#[test]
fn an_unknown_busyness_reads_as_not_known_yet_and_never_as_zero() {
    let unknown = json!({"processors": 8, "busy_percent_step": null, "window_seconds": 900});
    let known = json!({"processors": 8, "busy_percent_step": 30, "window_seconds": 900});

    assert_eq!(
        what(Family::Processors, &unknown),
        "8 processors, busy: not known yet"
    );
    assert_eq!(
        what(Family::Processors, &known),
        "8 processors, 30% busy over 15 min"
    );
    assert!(
        means(Family::Processors, &unknown)
            .join(" ")
            .contains("average over the last 15 minutes")
    );
}

#[test]
fn available_memory_is_shown_in_the_free_column() {
    let reading = resources();
    let pane = TheHostAndItsFiles.panes().remove(0);
    let rows = pane.rows(&reading, &Showing::default());
    let memory = rows
        .iter()
        .find(|row| row.key == "memory|summary")
        .expect("the memory row");

    let cells: Vec<String> = pane
        .cells(&reading, memory, Room::of(80))
        .into_iter()
        .map(|cell| cell.text)
        .collect();

    assert!(cells.contains(&"60%".to_string()), "{cells:?}");
    assert_eq!(
        free(Family::Memory, &json!({"available_percent_step": null})),
        "—"
    );
}
