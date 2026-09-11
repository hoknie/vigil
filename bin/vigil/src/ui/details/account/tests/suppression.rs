use super::harness::drawn;
use crate::ui::fixture;

#[test]
fn every_kind_of_object_spells_the_suppression_with_its_own_key_character_for_character() {
    for key in [
        "account|backdoor",
        "group|docker",
        "sudoer|%wheel",
        "sshkey|deploy|SHA256:3VaOaGZ8sBqrDLBz5nfCTd3bAqTL1s1a7uYRoOoJcVQ",
        "session|deploy|pts/0",
    ] {
        let page = drawn(&fixture::view(), key, 100);
        let unbroken: String = page.chars().filter(|c| !c.is_whitespace()).collect();

        assert!(page.contains("suppressions:"), "{key}: {page}");
        assert!(
            unbroken.contains(&format!("\"user|{key}\"")),
            "{key} is not spelled whole: {page}"
        );
    }
}
