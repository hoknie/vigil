pub const ROOM_FOR_THE_COLLECTOR: u16 = 118;

pub const MARKER: usize = 3;

pub const NUMBER: usize = 2;

pub const HEALTH: usize = 2;

pub const SECTION: &str = "SECTION";

pub const HOLDS: &str = "WHAT IT HOLDS";

pub const STATE: &str = "STATE";

pub const OBJECTS: &str = "OBJECTS";

pub const READ: &str = "READ";

pub const COLLECTOR: &str = "COLLECTOR";

const ROOM: usize = 62;

const STATE_WIDTH: usize = 11;

const OBJECTS_WIDTH: usize = 7;

const READ_WIDTH: usize = 8;

const COLLECTOR_WIDTH: usize = 31;

const NAME_LEAST: usize = SECTION.len();

const HOLDS_LEAST: usize = 25;

pub fn widths(wide: bool, longest: usize) -> Vec<(&'static str, usize)> {
    let for_the_two = ROOM - STATE_WIDTH - OBJECTS_WIDTH - READ_WIDTH;
    let name = longest.clamp(NAME_LEAST, for_the_two);
    let holds = for_the_two - name;

    let mut names = vec![(SECTION, name)];
    if holds >= HOLDS_LEAST {
        names.push((HOLDS, holds));
    }
    names.push((STATE, STATE_WIDTH));
    names.push((OBJECTS, OBJECTS_WIDTH));
    names.push((READ, READ_WIDTH));
    if wide {
        names.push((COLLECTOR, COLLECTOR_WIDTH));
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Screen;
    use crate::ui::helpers::layout::column;

    fn drawn(longest: usize) -> usize {
        MARKER + HEALTH + NUMBER + column::columns(&widths(false, longest)).chars().count()
    }

    #[test]
    fn the_table_fits_the_narrowest_terminal_this_console_is_read_on() {
        for longest in NAME_LEAST..=64 {
            assert!(
                drawn(longest) <= 78,
                "{} columns inside a panel eighty wide, for a name of {longest}",
                drawn(longest)
            );
        }
    }

    #[test]
    fn a_name_longer_than_the_column_widens_the_column_rather_than_losing_its_tail() {
        for longest in NAME_LEAST..=40 {
            let widths = widths(false, longest);
            let (_, name) = widths.first().expect("a column for the name");

            assert!(
                *name >= longest.min(ROOM - STATE_WIDTH - OBJECTS_WIDTH - READ_WIDTH),
                "a name of {longest} is drawn in a column of {name}, and the tail of it is a \
                 word the reader has to guess at"
            );
        }
    }

    #[test]
    fn what_a_section_holds_is_dropped_before_its_name_is_cut() {
        let roomy = widths(false, NAME_LEAST);
        let cramped = widths(false, 30);

        assert!(roomy.iter().any(|(named, _)| *named == HOLDS));
        assert!(
            !cramped.iter().any(|(named, _)| *named == HOLDS),
            "a name that long leaves no room for a sentence beside it, and half a sentence is \
             worse than none"
        );
        assert!(cramped.iter().any(|(named, _)| *named == STATE));
    }

    #[test]
    fn the_readings_of_every_section_are_named_whole_in_the_column_that_names_them() {
        for screen in Screen::all() {
            let named = screen.collectors().join(" \u{b7} ");
            assert!(
                named.chars().count() <= COLLECTOR_WIDTH,
                "{} reads {named:?}, which is {} characters in a column of {COLLECTOR_WIDTH}: \
                 the name of a reading cut in half is a reading the reader cannot look up",
                screen.name(),
                named.chars().count()
            );
        }
    }

    #[test]
    fn every_sentence_this_column_draws_fits_the_narrowest_it_is_ever_drawn_at() {
        assert!(
            super::super::rows::NO_SCREEN.chars().count() <= HOLDS_LEAST,
            "the row of a reading with no screen of its own is drawn in this column too"
        );
        for screen in Screen::all() {
            assert!(
                screen.holds().chars().count() <= HOLDS_LEAST,
                "{} holds {:?}, which is {} characters in a column of {HOLDS_LEAST}",
                screen.name(),
                screen.holds(),
                screen.holds().chars().count()
            );
        }
    }
}
