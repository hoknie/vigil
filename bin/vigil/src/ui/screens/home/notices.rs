pub const NO_SECTION: &str = "This console has no section for it: the agent is newer. Both ship in one package and \
     belong installed together.";

pub const NOT_WATCHED: &str =
    "This agent does not watch it: nothing in its configuration names that collector.";

pub const SEARCH_LIVES_IN_A_LIST: &str =
    "Search belongs to the list inside a section: open one and press / there.";

pub fn nothing_to_open(name: &str) -> String {
    format!("{name} has a row here and nothing behind it: this console cannot show that reading.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_console_that_cannot_show_a_reading_says_which_of_the_two_is_behind() {
        assert_ne!(
            NO_SECTION, NOT_WATCHED,
            "a section this console cannot draw and one the agent never reads are two \
             different hosts to go and look at"
        );
        assert!(nothing_to_open("resources").contains("resources"));
    }
}
