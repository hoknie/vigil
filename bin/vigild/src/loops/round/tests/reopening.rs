use vigil_collect::Health;
use vigil_store::Store;

use super::harness::{snapshot, watching};
use crate::types::Said;

fn handed_over(round: &crate::loops::Round, kind: &str) -> usize {
    round.shared.with(|state| {
        state
            .latest_findings(None)
            .iter()
            .filter(|finding| finding.kind.as_str() == kind)
            .count()
    })
}

#[test]
fn a_port_that_opens_and_closes_twice_is_news_all_four_times() {
    let mut it = watching(
        "flapping",
        Health::Ok,
        vec![
            snapshot(&[443]),
            snapshot(&[443, 4444]),
            snapshot(&[443]),
            snapshot(&[443, 4444]),
            snapshot(&[443]),
        ],
    );
    let mut said = Said::about([("network", "ok".to_string())]);

    for _ in 0..5 {
        it.round.read(0, &mut said);
    }

    assert_eq!(
        handed_over(&it.round, "port.listen.new"),
        2,
        "the second opening came after the first was closed, and a receiver that never hears \
         of it believes the port is still shut"
    );
    assert_eq!(
        handed_over(&it.round, "port.listen.removed"),
        2,
        "and the second closing closed something, so it is news too"
    );
    let reopened = it
        .round
        .store
        .open_finding("port.listen|tcp|0.0.0.0:4444", "port.listen.new")
        .expect("readable");
    assert!(
        reopened.is_none(),
        "after the second closing the opening is resolved again"
    );
}

#[test]
fn an_open_finding_seen_again_raises_its_counter_and_is_not_handed_over_twice() {
    let mut it = watching("repeating", Health::Ok, vec![snapshot(&[443])]);
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    let mut finding = crate::socket::fixture::finding("4444");
    finding.finding_key = "port.listen|tcp|0.0.0.0:4444".into();

    let first = it.round.remember(std::slice::from_ref(&finding));
    let second = it.round.remember(std::slice::from_ref(&finding));

    assert_eq!(first.len(), 1);
    assert!(
        second.is_empty(),
        "a repeat of something still open is a counter, not news"
    );
}

#[test]
fn a_closing_that_closed_nothing_is_not_handed_over_again() {
    let mut it = watching("closing-nothing", Health::Ok, vec![snapshot(&[443])]);
    let mut said = Said::about([("network", "ok".to_string())]);
    it.round.read(0, &mut said);
    let mut closing = crate::socket::fixture::finding("4444");
    closing.finding_key = "port.listen|tcp|0.0.0.0:4444".into();
    closing.kind = vigil_model::Kind::Known(vigil_model::KnownKind::PortListenRemoved);

    let first = it.round.remember(std::slice::from_ref(&closing));
    let second = it.round.remember(std::slice::from_ref(&closing));

    assert_eq!(first.len(), 1, "the first one is new");
    assert!(
        second.is_empty(),
        "the second closed nothing and repeats the first, so it is a counter"
    );
}
