use vigil_view::{Notice, Showing};

const NAMED_IN_THE_CONFIGURATION: &str = "The list is the paths the watch list names, each \
                                          directory walked whole and each mask matched, plus \
                                          the directories on PATH.";

pub(super) fn nothing_read() -> Notice {
    Notice::plain("No path is being watched on this host.")
        .saying(NAMED_IN_THE_CONFIGURATION)
        .saying(
            "A host with nothing named watches nothing: this is a configuration that asked \
             for none, not a reading that failed.",
        )
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    match showing.holding_back() {
        true => Notice::plain(format!(
            "No file and no directory matches {:?}.",
            showing.search
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        ),
        false => Notice::plain("This reading lists no file and no directory.")
            .saying(NAMED_IN_THE_CONFIGURATION),
    }
}
