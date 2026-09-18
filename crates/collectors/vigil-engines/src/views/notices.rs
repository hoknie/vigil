use vigil_model::Snapshot;
use vigil_view::{Notice, Showing};

use crate::types::{Engine, List, Standing, Subject};

pub(super) fn why(reading: &Snapshot, engine: Engine, list: List, showing: &Showing<'_>) -> Notice {
    let standing = Standing::in_reading(reading, engine);
    let name = engine.name();
    let from_the_files = list.subject() == Subject::Registry;

    if !standing.watched {
        return Notice::plain(format!("{name} is not watched on this host.")).saying(format!(
            "`engines:` in the containers block of vigil.yaml does not name {name}, so the \
             agent does not read what it holds."
        ));
    }
    if !from_the_files && !standing.dump_read {
        return Notice::loud(format!(
            "What {name} holds is unknown: its dump could not be read."
        ))
        .saying(
            "The summary screen says why, under the containers-engines collector. This is \
                 not an engine that holds nothing.",
        );
    }
    if !from_the_files && !standing.present {
        return Notice::plain(format!("{name} is not installed on this host.")).saying(format!(
            "vigil-container-dump looked for the {name} program and found none, so nothing \
             it would hold is listed."
        ));
    }
    if standing.silent_on(list.subject()) {
        return Notice::loud(format!("{name} did not answer for its {}.", list.name())).saying(
            "The command failed or did not finish when vigil-containers.timer last ran, so \
             what it holds is unknown rather than nothing. No finding says any of it was \
             removed.",
        );
    }

    empty(engine, list, showing)
}

pub(super) fn empty(engine: Engine, list: List, showing: &Showing<'_>) -> Notice {
    if !showing.search.is_empty() {
        return Notice::plain(format!(
            "No {} of {} matches {:?}.",
            list.thing(),
            engine.name(),
            showing.search
        ))
        .saying(
            "The search covers every value recorded about the row. Press / to change it, Esc \
             to drop it.",
        );
    }
    if !showing.only.is_empty() {
        return Notice::plain(format!(
            "No {} of {} is left once the list is narrowed.",
            list.thing(),
            engine.name()
        ))
        .saying("Press f and choose everything to see the whole list again.");
    }

    if list == List::Registries {
        return Notice::plain(format!(
            "No registry is written for {} in {}.",
            engine.name(),
            engine.registries_file()
        ))
        .saying("The engine pulls from its own default registry and from nowhere else named.");
    }

    Notice::plain(format!("{} holds no {}s.", engine.name(), list.thing())).saying(
        "The engine answered and listed none: this is an engine with none, not a reading that \
         failed.",
    )
}
