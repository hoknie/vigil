use super::fields::{hooked_on_input, legacy_backend};
use super::rows::{COLLECTOR, summary};
use super::showing::Showing;
use crate::ui::helpers::words::{moment, refusal};
use crate::ui::{Gone, Notice, Reading, View};

const WRITTEN_BY: &str = "The reading is written by the vigil-firewall.timer unit and read from a file; the agent starts no program of its own.";

const NOT_THE_SAME: &str =
    "What this host filters is unknown, which is not the same as a host that filters nothing.";

pub(super) fn missing(view: &View) -> Option<Notice> {
    if view.switched_off(COLLECTOR) {
        return Some(
            Notice::plain("This agent is not reading what the host lets in.")
                .saying(
                    view.collector_reason(COLLECTOR)
                        .unwrap_or("switched off in the configuration")
                        .to_string(),
                )
                .saying("The console asks for no reading it was told is switched off."),
        );
    }

    match view.reading(COLLECTOR) {
        Reading::Unknown => Some(
            Notice::plain("The agent has not been asked yet.")
                .saying("Press r to ask now; otherwise every couple of seconds."),
        ),
        Reading::NotTakenYet => Some(
            Notice::plain("The agent has not read the firewall yet.")
                .saying("The first reading is due within the period on the summary screen."),
        ),
        Reading::Refused(refusal) => Some(
            refusal::refused(
                refusal,
                COLLECTOR,
                "No table, chain or policy is listed here: nothing was read.",
            )
            .saying(NOT_THE_SAME)
            .saying(WRITTEN_BY),
        ),
        Reading::Taken(snapshot) if snapshot.items.is_empty() => Some(
            Notice::loud(format!(
                "The reading taken at {} holds nothing at all.",
                moment::time_of_day(&snapshot.taken_at)
            ))
            .saying(
                "Even a host with no tables is read as a ruleset with none in it, so a reading \
                 with no rows at all is a failed one.",
            )
            .saying(NOT_THE_SAME),
        ),
        Reading::Taken(_) => None,
    }
}

pub(super) fn empty(view: &View, showing: &Showing<'_>) -> Notice {
    if showing.search.holding_back() {
        return Notice::plain(format!(
            "No table, chain or marker matches {:?}.",
            showing.search.query()
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        );
    }

    Notice::plain("This reading lists no table and no chain.")
        .saying(what_that_means(view))
        .saying(NOT_THE_SAME)
}

pub(super) fn gone(gone: &Gone) -> Notice {
    Notice::loud(format!("{} is not in this reading any more.", gone.key))
        .saying(format!(
            "The finding said: {} ({}), and the agent last had that row in front of it at {}.",
            gone.title,
            gone.kind,
            moment::time_of_day(&gone.last_seen)
        ))
        .saying(
            "A finding about a flushed table or a host that stopped filtering is a finding about \
         a row that is gone by the time it is opened: that is the event itself, not a failure \
         to find it. What the reading holds now is under this.",
        )
}

pub(super) fn about(view: &View) -> Vec<String> {
    let Some(item) = summary(view) else {
        return Vec::new();
    };
    vec![counted(item), what_that_means(view)]
}

fn counted(item: &serde_json::Value) -> String {
    use super::fields::{all_chains, all_rules, base_chains, families, tables};

    let families = families(item);
    let named = match families.is_empty() {
        true => "no family".to_string(),
        false => format!("families {}", families.join(", ")),
    };

    format!(
        "{} table(s) · {} chain(s), {} of them on a hook · {} rule(s) · {named}",
        tables(item),
        all_chains(item),
        base_chains(item),
        all_rules(item)
    )
}

fn what_that_means(view: &View) -> String {
    let Some(item) = summary(view) else {
        return NOT_THE_SAME.to_string();
    };

    if legacy_backend(item) {
        return "nftables holds nothing here and the legacy iptables backend has tables \
                registered: the rules are in the old backend, which this build reads as a \
                marker and does not parse. This is not a host without a firewall."
            .to_string();
    }

    match hooked_on_input(item) {
        0 => "No chain is on the input hook: nothing in this ruleset drops an incoming packet."
            .to_string(),
        1 => "1 chain is on the input hook, and its policy is what happens to a packet no rule \
              allowed."
            .to_string(),
        many => format!(
            "{many} chains are on the input hook; each one's policy is what happens to a packet \
             no rule in it allowed."
        ),
    }
}
