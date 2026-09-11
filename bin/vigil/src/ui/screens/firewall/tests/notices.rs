use vigil_model::CollectorState;

use super::harness::{drawn, holding, refused, squashed};
use crate::ui::fixture;

const NO_FILE: &str = "/var/lib/vigil/firewall/ruleset.json is not there, so what this host filters is unknown — which is not the same as nothing. Either the timer has never run (systemctl enable --now vigil-firewall.timer), it is masked, or /usr/sbin/nft is not installed here";

const NOTHING_WRITTEN: &str = "could not parse /var/lib/vigil/firewall/ruleset.json: the ruleset file is empty: nft was started and wrote nothing, so what this host filters is unknown";

const NOT_JSON: &str = "could not parse /var/lib/vigil/firewall/ruleset.json: the ruleset file is not JSON, it begins \"Error: could not process rule: Operation not permitted\": what this host filters is unknown";

const TOO_OLD: &str = "the ruleset in /var/lib/vigil/firewall/ruleset.json was written 214 seconds ago, more than the 120 this collector allows: what it says about this host may have been true and no longer is";

fn four_ways_the_file_can_fail() -> [(CollectorState, &'static str, &'static str); 4] {
    [
        (CollectorState::Unavailable, NO_FILE, "is not there"),
        (CollectorState::Degraded, NOTHING_WRITTEN, "wrote nothing"),
        (CollectorState::Degraded, NOT_JSON, "is not JSON"),
        (CollectorState::Degraded, TOO_OLD, "214 seconds ago"),
    ]
}

#[test]
fn a_file_that_could_not_be_read_is_a_named_screen_and_never_an_empty_table() {
    let mut said: Vec<String> = Vec::new();

    for (state, reason, distinctive) in four_ways_the_file_can_fail() {
        let page = drawn(&refused(state, reason), 80);

        assert!(
            squashed(&page).contains(&squashed(distinctive)),
            "the agent's own words for this refusal are not on the screen: {page}"
        );
        assert!(
            !page.contains("POLICY"),
            "a table with no rows under it reads as a host with no rules: {page}"
        );
        assert!(
            squashed(&page).contains(&squashed("No table, chain or policy is listed here")),
            "{page}"
        );
        assert!(
            squashed(&page).contains(&squashed("not the same as a host that filters nothing")),
            "every one of the four has to say this, or the screen answers 'there is no \
             attack': {page}"
        );
        said.push(page);
    }

    for (index, page) in said.iter().enumerate() {
        for (other, another) in said.iter().enumerate() {
            if index != other {
                assert_ne!(page, another, "two of the four read the same on the screen");
            }
        }
    }
}

#[test]
fn a_refusal_says_which_unit_writes_the_reading_so_a_reader_knows_what_to_go_and_look_at() {
    let page = drawn(&refused(CollectorState::Unavailable, NO_FILE), 80);

    assert!(page.contains("vigil-firewall.timer"), "{page}");
    assert!(
        squashed(&page).contains(&squashed("starts no program of its own")),
        "the reader is told where the privilege lives, because that is what they will \
         switch on or off: {page}"
    );
}

#[test]
fn a_reading_this_build_cannot_take_is_told_apart_from_a_host_with_no_rules() {
    let cannot_read = drawn(&refused(CollectorState::Degraded, NOTHING_WRITTEN), 80);
    let reads_nothing = drawn(&holding(fixture::firewall::filtering_nothing()), 80);

    assert!(
        squashed(&cannot_read).contains(&squashed("nothing was read")),
        "{cannot_read}"
    );
    assert!(
        squashed(&reads_nothing).contains(&squashed("No chain is on the input hook")),
        "a host that really filters nothing is a reading, and it says what the reading \
         means: {reads_nothing}"
    );
    assert!(
        !squashed(&reads_nothing).contains(&squashed("nothing was read")),
        "{reads_nothing}"
    );
}

#[test]
fn a_host_whose_rules_the_old_backend_holds_is_never_drawn_as_a_host_without_a_firewall() {
    let page = drawn(&holding(fixture::firewall::held_by_the_old_backend()), 80);

    assert!(page.contains("legacy iptables"), "{page}");
    assert!(
        squashed(&page).contains(&squashed("This is not a host without a firewall")),
        "the marker has to say in words what it means, or an empty table speaks for it: \
         {page}"
    );
    assert!(
        squashed(&page).contains(&squashed("filter, nat")),
        "and which tables the old backend registered: {page}"
    );
}
