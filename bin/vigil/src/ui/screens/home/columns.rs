pub const ROOM_FOR_THE_COLLECTOR: u16 = 118;

pub const MARKER: usize = 3;

pub const NUMBER: usize = 2;

pub fn widths(wide: bool) -> Vec<(&'static str, usize)> {
    let mut names = vec![
        ("SECTION", 9),
        ("WHAT IT HOLDS", 26),
        ("STATE", 11),
        ("OBJECTS", 7),
        ("READ", 8),
    ];
    if wide {
        names.push(("COLLECTOR", 12));
    }
    names
}

#[cfg(test)]
mod tests {
    use crate::ui::helpers::layout::column;

    use super::*;

    #[test]
    fn the_table_fits_the_narrowest_terminal_this_console_is_read_on() {
        let header = column::columns(&widths(false));

        assert!(
            MARKER + NUMBER + header.chars().count() <= 78,
            "{} columns inside a panel eighty wide",
            MARKER + NUMBER + header.chars().count()
        );
    }
}
