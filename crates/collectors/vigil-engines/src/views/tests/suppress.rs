use vigil_model::Snapshot;
use vigil_rules::{RuleContext, diff};
use vigil_view::{Pane, Piece, RowKey, Section, Showing};

use super::super::WhatTheEnginesHold;
use crate::fixture;
use crate::rules::engine_rules;
use crate::types::Report;

fn printed(pane: &dyn Pane, reading: &Snapshot, key: &str) -> Option<String> {
    pane.detail(reading, &RowKey::of(key), 80)
        .into_iter()
        .find_map(|piece| match piece {
            Piece::Key(key) => Some(key),
            _ => None,
        })
}

fn raised_when_it_arrives(reading: &Snapshot, key: &str) -> Vec<String> {
    let mut before = reading.clone();
    before.items.remove(key);
    let mut mint = || "event".to_string();
    let mut ctx = RuleContext {
        now: "2026-09-18T12:00:00.000Z".into(),
        mint_event_id: &mut mint,
    };
    engine_rules(&Report::default())
        .judge(&diff(&before, reading), &mut ctx)
        .into_iter()
        .map(|finding| finding.finding_key)
        .collect()
}

#[test]
fn every_row_offers_to_be_suppressed_by_the_very_key_the_rules_raise_about_it() {
    let reading = fixture::engines();
    let mut compared = 0;

    for pane in WhatTheEnginesHold.panes() {
        assert!(
            pane.offers().suppressing,
            "{}: every row of these lists is something a rule reports on",
            pane.name()
        );
        for row in pane.rows(&reading, &Showing::default()) {
            let printed = printed(pane.as_ref(), &reading, &row.key)
                .unwrap_or_else(|| panic!("{} writes no key to suppress", row.key));
            let raised = raised_when_it_arrives(&reading, &row.key);
            for key in &raised {
                assert_eq!(
                    key, &printed,
                    "{}: a suppression written from the screen that does not carry the key \
                     the rule raises silences nothing, and the reader believes it did",
                    row.key
                );
                compared += 1;
            }
        }
    }

    assert!(
        compared >= 20,
        "only {compared} rows raised something, so this guard reads little"
    );
}
