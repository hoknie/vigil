use super::verdict::{judged, said};
use crate::fixture;
use crate::types::Subject;

#[test]
fn an_engine_that_did_not_answer_for_a_subject_has_not_removed_everything_it_held() {
    let before = fixture::engines();

    for subject in [
        Subject::Image,
        Subject::Volume,
        Subject::Network,
        Subject::Container,
    ] {
        let silent = fixture::docker_silent_on(subject);
        assert!(
            silent.items.len() < before.items.len(),
            "{subject:?}: the sample must lose rows for this test to mean anything"
        );

        assert_eq!(
            said(&judged(&before, &silent)),
            Vec::new(),
            "{subject:?}: a command that failed is a hole in the reading, and reporting every \
             row behind it as removed turns one refused socket into a page of findings"
        );
        assert_eq!(
            said(&judged(&silent, &before)),
            Vec::new(),
            "{subject:?}: and what comes back when the engine answers again is what was there, \
             not a page of new things"
        );
    }
}

#[test]
fn an_engine_uninstalled_or_installed_between_two_readings_is_learned_and_not_reported_row_by_row()
{
    let both = fixture::engines();
    let docker_only = fixture::only_docker();

    assert_eq!(said(&judged(&both, &docker_only)), Vec::new());
    assert_eq!(
        said(&judged(&docker_only, &both)),
        Vec::new(),
        "an engine's first answer is its baseline, as a collector's first reading is: the \
         pod, the secret and the volume bound to /etc were there before the engine was asked"
    );
}
