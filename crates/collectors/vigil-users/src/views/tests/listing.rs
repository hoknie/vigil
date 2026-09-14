use vigil_view::Showing;

use super::host::a_host_with_names_that_prefix_one_another as host;
use super::walks;
use crate::fixture::users;
use crate::types::Subject;
use crate::views::rows::rows;

#[test]
fn every_list_holds_the_rows_a_walk_over_the_whole_reading_held_and_in_the_same_order() {
    for reading in [host(), users()] {
        for subject in Subject::ALL.iter().copied() {
            for search in [
                "",
                "deploy",
                "deploy2",
                "ghost",
                "sha256",
                "nothing matches this",
            ] {
                let showing = Showing::searching(search);
                let found: Vec<String> = rows(&reading, subject, &showing)
                    .into_iter()
                    .map(|row| row.key)
                    .collect();

                assert_eq!(
                    found,
                    walks::rows(&reading, subject, &showing),
                    "{subject:?} searching {search:?}: a list reads only the range of its own \
                     kinds and looks an account up by its key, and must hold what the walk held"
                );
            }
        }
    }
}
