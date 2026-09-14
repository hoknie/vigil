use vigil_view::{Pane, Showing, conformance};

use super::searching::{SEARCHED, scaled};
use crate::types::Subject;
use crate::views::pane::Of;

#[test]
fn every_footer_written_from_the_counts_says_word_for_word_what_the_footer_from_the_reading_says() {
    let reading = scaled();

    for subject in Subject::ALL.iter().copied() {
        conformance::the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_also(
            &Of(subject),
            &reading,
            Showing::default(),
            SEARCHED,
        );
    }
}

#[test]
fn the_scaled_reading_gives_every_clause_of_the_footer_something_to_say() {
    let reading = scaled();

    for (subject, clause) in [
        (Subject::Users, "could log in"),
        (Subject::Users, "whose password state could not be read"),
        (Subject::Groups, "that grant power"),
        (Subject::Keys, "the agent was refused"),
        (Subject::SshUsers, "the agent was refused"),
        (Subject::LoggedIn, "with somebody at a terminal"),
        (Subject::LoggedIn, "logind.1: read, 2"),
        (Subject::LoggedIn, "utmp.1: not on this host"),
    ] {
        let said = Of(subject).tally(&reading, &Showing::searching("deploy"), 0);

        assert!(
            said.contains(clause),
            "{subject:?}: a clause the scaled reading never makes the footer say is a clause the \
             comparison with the counts never checks: {said}"
        );
    }
}
