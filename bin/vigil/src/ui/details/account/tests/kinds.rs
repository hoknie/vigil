use super::harness::drawn;
use crate::ui::fixture;

#[test]
fn a_groups_detail_says_why_it_is_privileged_and_who_is_in_it() {
    let page = drawn(&fixture::view(), "group|docker", 80);

    assert!(page.contains("DOCKER"), "{page}");
    assert!(page.contains("may start a container"), "{page}");
    assert!(page.contains("deploy"), "{page}");
}

#[test]
fn a_sudo_grants_detail_names_the_file_and_who_it_reaches() {
    let page = drawn(&fixture::view(), "sudoer|%wheel", 80);

    assert!(page.contains("%WHEEL"), "{page}");
    assert!(page.contains("/etc/sudoers"), "{page}");
    assert!(page.contains("ALL=(ALL) ALL"), "{page}");
    assert!(
        page.contains("deploy"),
        "the account it reaches through the group: {page}"
    );
}

#[test]
fn a_keys_detail_carries_the_whole_fingerprint() {
    let page = drawn(
        &fixture::view(),
        "sshkey|deploy|SHA256:3VaOaGZ8sBqrDLBz5nfCTd3bAqTL1s1a7uYRoOoJcVQ",
        80,
    );

    assert!(
        page.contains("SHA256:3VaOaGZ8sBqrDLBz5nfCTd3bAqTL1s1a7uYRoOoJcVQ"),
        "a cut fingerprint compares equal to the wrong key: {page}"
    );
    assert!(page.contains("person@laptop"), "{page}");
    assert!(page.contains("no command, source or expiry"), "{page}");
}
