use serde_json::json;
use vigil_model::Severity;

use super::verdict::{altered, judged, said, with, without};
use crate::fixture::{self, TAGGED_IMAGE};
use crate::types::{Engine, Subject};

const MOVED_TO: &str = "sha256:d00d5e1f0a42";

fn one(kind: &str, key: &str) -> Vec<(String, String)> {
    vec![(kind.to_string(), key.to_string())]
}

#[test]
fn a_tag_that_moved_to_another_image_is_one_changed_image_and_not_a_new_one_and_a_bare_one() {
    let before = fixture::engines();
    let pulled = with(
        &altered(
            &before,
            Engine::Docker,
            Subject::Image,
            "sha256:c4b2e1f09a33",
            "tags",
            json!([]),
        ),
        Engine::Docker,
        Subject::Image,
        MOVED_TO,
        json!({"subject": "image", "id": MOVED_TO, "tags": ["shop/api:2026.09.1"], "untagged": false, "digest": null, "size": "415MB"}),
    );
    let pruned = without(
        &pulled,
        Engine::Docker,
        Subject::Image,
        "sha256:c4b2e1f09a33",
    );

    for after in [&pulled, &pruned] {
        let findings = judged(&before, after);
        assert_eq!(
            said(&findings),
            one(
                "container.image.changed",
                &format!("engine|docker|image|{MOVED_TO}")
            ),
            "a deploy that pulled a new build of a tag is one event, and three findings about \
             it are the noise this product dies of"
        );
        assert_eq!(findings[0].severity, Severity::Medium);
        assert!(
            findings[0].evidence.iter().any(
                |one| one.value.contains("sha256:c4b2e1f09a33") && one.value.contains(MOVED_TO)
            ),
            "{:?}",
            findings[0].evidence
        );
    }
    assert_ne!(TAGGED_IMAGE, "sha256:c4b2e1f09a33");
}
