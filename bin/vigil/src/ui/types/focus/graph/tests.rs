use std::time::{Duration, Instant};

use vigil_view::{Piece, RowKey};

use super::{DRAWN, Graph, KEPT};

fn opened() -> Graph {
    Graph::open(RowKey::of("fw-interface|eth0"), vec![Piece::line("   in")])
}

fn at(seconds: u64) -> Instant {
    Instant::now() + Duration::from_secs(seconds)
}

#[test]
fn a_graph_counts_nothing_until_a_reader_asks_it_to_and_stops_when_the_reader_leaves() {
    let mut graph = opened();

    assert!(!graph.watching());
    let said = format!("{:?}", graph.pieces(at(0), Some(10)));
    assert!(said.contains("Counting is off"), "{said}");

    graph.watch(at(0));
    graph.note(Some(10));
    assert!(graph.watching() && graph.samples.len() == 1);

    graph.watch(at(1));
    assert!(
        !graph.watching() && graph.samples.is_empty(),
        "a panel that keeps its samples after it stopped keeps them for as long as the \
         console runs, and nothing ever looks at them again"
    );
}

#[test]
fn a_reading_that_did_not_move_is_not_a_second_sample() {
    let mut graph = opened();
    graph.watch(at(0));

    for _ in 0..20 {
        graph.note(Some(500));
    }

    assert_eq!(
        graph.samples.len(),
        1,
        "the console asks every two seconds and the host reads every sixty, so all but one of \
         those answers is the same answer told again"
    );
}

#[test]
fn a_panel_left_open_for_a_day_keeps_the_same_number_of_readings_as_one_just_opened() {
    let mut graph = opened();
    graph.watch(at(0));

    for step in 0..10_000u64 {
        graph.note(Some(step * 7));
    }

    assert_eq!(
        graph.samples.len(),
        KEPT,
        "a console left on a wall keeps this panel open for weeks, and a list that grows with \
         every reading is a console that ends by being killed for its memory"
    );
    assert_eq!(graph.samples.back(), Some(&(9_999 * 7)));
}

#[test]
fn nothing_is_counted_while_the_panel_is_not_watching() {
    let mut graph = opened();

    graph.note(Some(10));
    graph.note(Some(20));

    assert!(graph.samples.is_empty());
}

#[test]
fn a_host_that_counts_nothing_is_told_why_rather_than_shown_a_rate_of_zero() {
    let mut graph = opened();
    graph.watch(at(0));

    let said = format!("{:?}", graph.pieces(at(2), None));

    assert!(said.contains("not counting what goes through"), "{said}");
    assert!(!said.contains("a second,"), "{said}");
}

#[test]
fn the_rate_waits_for_a_second_reading_rather_than_dividing_by_the_first() {
    let mut graph = opened();
    graph.watch(at(0));
    graph.note(Some(1_000));

    let said = format!("{:?}", graph.pieces(at(2), Some(1_000)));

    assert!(said.contains("waiting for a second reading"), "{said}");
}

#[test]
fn a_rate_is_not_worked_out_from_less_than_a_second_of_watching() {
    let mut graph = opened();
    graph.watch(Instant::now());
    graph.note(Some(1_000));
    graph.note(Some(9_000));

    let said = format!("{:?}", graph.pieces(Instant::now(), Some(9_000)));

    assert!(
        said.contains("waiting for a second reading"),
        "eight thousand packets divided by the millisecond since the key was pressed is a \
         number no host ever did: {said}"
    );
}

#[test]
fn the_bar_draws_the_steps_between_readings_and_never_more_than_it_has_room_for() {
    let mut graph = opened();
    graph.watch(at(0));
    for step in 0..KEPT as u64 {
        graph.note(Some(step * step));
    }

    let bar = graph.bar().expect("the steps are drawn");

    assert_eq!(bar.chars().count(), DRAWN);
    assert_eq!(
        bar.chars().last(),
        Some('█'),
        "the newest step is on the right, where a reader's eye lands"
    );
}

#[test]
fn a_link_nothing_went_through_draws_no_bar_at_all() {
    let mut graph = opened();
    graph.watch(at(0));
    graph.note(Some(7));

    assert_eq!(
        graph.bar(),
        None,
        "a row of the lowest block drawn for a link with no traffic reads as traffic"
    );
}

#[test]
fn a_panel_scrolled_past_its_end_stops_at_the_last_page() {
    let mut graph = opened();

    for _ in 0..50 {
        graph.scroll(crate::ui::Motion::Down, 10, 30);
    }

    assert_eq!(graph.top(), 20);
}
