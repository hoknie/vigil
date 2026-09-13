use vigil_view::{Notice, Showing};

pub(super) fn nothing_read() -> Notice {
    Notice::plain("Nothing is running in a container on this host.").saying(
        "The reading was taken and holds no container and no runtime socket: this is a host \
         with none, not a reading that failed.",
    )
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    if showing.holding_back() {
        return Notice::plain(format!(
            "No container and no socket matches {:?}.",
            showing.search
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        );
    }

    Notice::plain("Nothing is running in a container on this host.").saying(
        "The reading was taken and holds no container and no runtime socket: this is a host \
         with none, not a reading that failed.",
    )
}
