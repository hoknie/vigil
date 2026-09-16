use vigil_model::Changing;

use super::hints::Hints;
use crate::ui::{Level, Screen};

pub(super) fn keys(hints: &Hints<'_>, screen: Screen, width: u16) -> String {
    if hints.editing {
        return [FORM, FORM_SHORT, LAST_TWO_OF_A_FORM]
            .into_iter()
            .find(|line| line.chars().count() <= width as usize)
            .unwrap_or(LAST_TWO_OF_A_FORM)
            .to_string();
    }
    if hints.history {
        return [HISTORY, HISTORY_SHORT, LAST_TWO_OF_A_FORM]
            .into_iter()
            .find(|line| line.chars().count() <= width as usize)
            .unwrap_or(LAST_TWO_OF_A_FORM)
            .to_string();
    }
    if hints.typing {
        return " type to search · Enter keep it · Esc put it back".to_string();
    }
    if hints.choosing {
        return match hints.choosing_acts {
            true => " a letter above does it, on this host, now · C or Esc walks away".to_string(),
            false => " ← → choose · Enter apply · Esc leave it as it was".to_string(),
        };
    }

    let long = match hints.level {
        _ if screen == Screen::HOME => match hints.panel {
            true => " j/k ↑↓ a section · → or Enter open it · d close · 1-9 by number · ? keys"
                .to_string(),
            false => " j/k ↑↓ a section · → or Enter open it · d details · 1-9 by number · ? keys"
                .to_string(),
        },
        Level::Menu => format!(
            " ←→ which list · ↓ into it · ↑ or Esc {} · ? keys · q quit",
            hints.back.named()
        ),
        Level::Detail if hints.buttons => format!(
            " j/k ↑↓ scroll · → a button · Enter press it · ← or Esc {} · ? keys",
            BACK_TO_THE_LIST
        ),
        Level::Detail if hints.to_object => {
            " j/k ↑↓ PgUp/PgDn scroll · ← or Esc back to the list · o object · ? keys".to_string()
        }
        Level::Detail => {
            " j/k ↑↓ PgUp/PgDn scroll · ← or Esc back to the list · ? keys · q quit".to_string()
        }
        Level::List if screen == Screen::SUMMARY => format!(
            " j/k ↑↓ scroll · d {} · ← or Esc {} · r ask · ? keys",
            match hints.panel {
                true => "hide",
                false => "why",
            },
            hints.back.named()
        ),
        Level::List if hints.panel && hints.marks => {
            listing(hints, Room::Whole, CLOSE_THE_PANEL, Picking::Unsaid)
        }
        Level::List if hints.panel => {
            " j/k ↑↓ move · → detail · / search · ← or Esc close the panel · ? keys".to_string()
        }
        Level::List if screen == Screen::FINDINGS => {
            listing(hints, Room::Whole, hints.back.named(), Picking::AndOne)
        }
        Level::List => listing(hints, Room::Whole, hints.back.named(), Picking::Unsaid),
    };
    let back = match hints.panel {
        true => CLOSE_THE_PANEL,
        false => hints.back.named(),
    };

    let offered = [
        Some(long),
        picking(hints, screen, Picking::Runs),
        picking(hints, screen, Picking::Unsaid),
        short(hints, screen),
        Some(listing(hints, Room::Tight, back, Picking::Unsaid)),
        Some(listing(hints, Room::Cramped, back, Picking::Unsaid)),
    ];
    for line in offered.into_iter().flatten() {
        if line.chars().count() <= width as usize {
            return line;
        }
    }
    LAST_TWO.to_string()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Room {
    Whole,
    Tight,
    Cramped,
}

pub(super) const LAST_TWO: &str = " ? keys · q quit";

const FORM: &str = " ↑↓ Tab a field · Space switch · ←→ a choice · Enter on a button · Esc back";

const FORM_SHORT: &str = " ↑↓ a field · Enter on a button · Esc back";

const LAST_TWO_OF_A_FORM: &str = " Esc back";

const HISTORY: &str = " j/k ↑↓ PgUp/PgDn scroll · ← or Esc back to the list · q quit";

const HISTORY_SHORT: &str = " ↑↓ scroll · Esc back to the list";

fn changes(offered: &[Changing], room: Room) -> String {
    if offered.is_empty() {
        return String::new();
    }
    let named = |changing: &Changing| match changing {
        Changing::Create => ('n', "new"),
        Changing::Update => ('e', "edit"),
        Changing::Delete => ('D', "delete"),
    };
    match room {
        Room::Cramped => format!(
            " · {} change",
            offered
                .iter()
                .map(|changing| named(changing).0.to_string())
                .collect::<Vec<String>>()
                .join("/")
        ),
        _ => offered
            .iter()
            .map(|changing| {
                let (key, word) = named(changing);
                format!(" · {key} {word}")
            })
            .collect(),
    }
}

const CLOSE_THE_PANEL: &str = "close the panel";

const BACK_TO_THE_LIST: &str = "back to the list";

fn short(hints: &Hints<'_>, screen: Screen) -> Option<String> {
    match hints.level {
        Level::List if screen == Screen::FINDINGS && !hints.panel => Some(format!(
            " j/k ↑↓ move · → detail · s sort · f filter · ← {} · ? keys",
            hints.back.named()
        )),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Picking {
    AndOne,
    Runs,
    Unsaid,
}

fn picking(hints: &Hints<'_>, screen: Screen, picking: Picking) -> Option<String> {
    match hints.level {
        Level::List if screen == Screen::FINDINGS && !hints.panel => {
            Some(listing(hints, Room::Whole, hints.back.named(), picking))
        }
        _ => None,
    }
}

fn listing(hints: &Hints<'_>, room: Room, back: &str, picking: Picking) -> String {
    let mut line = String::from(" j/k ↑↓ move");
    line.push_str(match picking {
        Picking::AndOne => " · ⇧↑↓ pick · x one",
        Picking::Runs => " · ⇧↑↓ pick",
        Picking::Unsaid => "",
    });
    if room != Room::Cramped {
        line.push_str(" · → detail");
    }
    if let Some(key) = hints.arranges
        && room == Room::Whole
    {
        line.push_str(&format!(" · {key} view"));
    }
    if hints.histories {
        line.push_str(" · H history");
    }
    if hints.marks {
        line.push_str(" · x mark");
        if hints.kills {
            line.push_str(match hints.stops {
                true => " · K stop",
                false => " · K close",
            });
        }
        line.push_str(&changes(hints.changes, room));
        if room == Room::Whole {
            line.push_str(" · S suppress");
        }
    }
    if hints.sorts && room != Room::Cramped {
        line.push_str(" · s sort");
    }
    if hints.filters {
        line.push_str(" · f filter");
    }
    if room == Room::Whole {
        line.push_str(" · / search");
    }
    line.push_str(&format!(" · ← {back} · ? keys"));
    line
}
