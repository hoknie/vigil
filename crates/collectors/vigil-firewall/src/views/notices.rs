use vigil_model::Snapshot;
use vigil_view::{Notice, Showing};

use super::fields::{hooked_on_input, legacy_backend};
use super::rows::summary;

#[cfg(not(target_os = "macos"))]
const WRITTEN_BY: &str = "The reading is written by the vigil-firewall.timer unit and read from a file; the agent starts no program of its own.";

#[cfg(target_os = "macos")]
const WRITTEN_BY: &str = "The reading is written by the launchd job vigil.firewall and read from a file; the agent starts no program of its own.";

const NOT_THE_SAME: &str =
    "What this host filters is unknown, which is not the same as a host that filters nothing.";

pub(super) fn nothing_read() -> Notice {
    Notice::loud("The reading holds nothing at all.")
        .saying(
            "Even a host with no tables is read as a ruleset with none in it, so a reading \
             with no rows at all is a failed one.",
        )
        .saying(NOT_THE_SAME)
        .saying(WRITTEN_BY)
}

pub(super) fn empty(reading: &Snapshot, showing: &Showing<'_>) -> Notice {
    if showing.holding_back() {
        return Notice::plain(format!(
            "No table, chain or marker matches {:?}.",
            showing.search
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        );
    }

    Notice::plain("This reading lists no table and no chain.")
        .saying(what_that_means(reading))
        .saying(NOT_THE_SAME)
}

fn what_that_means(reading: &Snapshot) -> String {
    let Some(item) = summary(reading) else {
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
