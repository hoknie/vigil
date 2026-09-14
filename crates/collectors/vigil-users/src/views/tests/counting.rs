use vigil_view::{Pane, Showing};

use super::searching::scaled;
use crate::types::Subject;
use crate::views::pane::Of;

const SEARCHES: &[&str] = &[
    "",
    "a",
    "d",
    "e",
    "s",
    "u",
    "0",
    "|",
    ".",
    "%",
    "deploy",
    "deploy.1",
    "DEPLOY2",
    "Root",
    "sha256:",
    "unreadable",
    "utmp",
    "logind",
    "pts/",
    "session-source",
    "nothing on this host reads like this",
];

#[test]
fn every_footer_written_from_the_counts_says_word_for_word_what_the_footer_from_the_reading_says() {
    let reading = scaled();

    for subject in Subject::ALL.iter().copied() {
        let pane = Of(subject);
        let counts = pane
            .counts(&reading, &Showing::default())
            .expect("every accounts list counts its reading once");

        for search in SEARCHES {
            for note in [None, Some("part of the reading was refused")] {
                for elsewhere in [0, 3] {
                    let showing = Showing {
                        elsewhere,
                        ..Showing::searching(search).noting(note)
                    };
                    let rows = pane.rows(&reading, &showing);

                    assert_eq!(
                        pane.tally_listed(&reading, &showing, &rows, &counts),
                        pane.tally(&reading, &showing, rows.len()),
                        "{subject:?} searching {search:?} with {note:?} and {elsewhere} other \
                         list(s): the console writes the footer from what it counted once and \
                         the rows it lists, and a reader must not be able to tell it from the \
                         footer that walked the whole reading"
                    );
                }
            }
        }
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
