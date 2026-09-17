use vigil_view::{Notice, Showing};

const NOT_THE_SAME: &str = "What this host is running on is unknown, which is not the same as a host with room to \
     spare.";

pub(super) fn nothing_read() -> Notice {
    Notice::loud("This reading holds nothing at all.")
        .saying(
            "Every host has a boot and a memory, so a reading with no row at all is a failed \
             one.",
        )
        .saying(NOT_THE_SAME)
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    match showing.holding_back() {
        true => Notice::plain(format!(
            "No boot, memory or filesystem matches {:?}.",
            showing.search
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        ),
        false => Notice::plain("This reading lists nothing at all.").saying(NOT_THE_SAME),
    }
}

pub(super) fn nothing_on_a_store(showing: &Showing<'_>) -> Notice {
    match showing.holding_back() {
        true => Notice::plain(format!("No filesystem matches {:?}.", showing.search)).saying(
            "The search covers every value recorded about the filesystem, the store it is \
             written to among them. Press / to change it, Esc to drop it.",
        ),
        false => Notice::plain("This host has no filesystem this agent can measure.").saying(
            "What a filesystem is written to is read from the block devices this kernel names; \
             a host that names none is gathered under whatever mounted each one.",
        ),
    }
}
