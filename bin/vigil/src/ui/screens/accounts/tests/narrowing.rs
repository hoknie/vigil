use super::harness::{drawn_seeking, looking_for};
use crate::ui::{Subject, fixture};

#[test]
fn a_search_narrows_the_list_it_was_typed_into_and_says_that_it_is_that_lists_own() {
    let view = fixture::view();

    let groups = drawn_seeking(&view, Subject::Groups, &looking_for("docker"), 80);
    assert!(groups.contains("docker"), "{groups}");
    assert!(!groups.contains("wheel"), "{groups}");

    let users = drawn_seeking(&view, Subject::Users, &looking_for("docker"), 80);
    assert!(users.contains("search"), "{users}");
    assert!(users.contains("No account matches"), "{users}");
    let unbroken = users.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(unbroken.contains("belongs to this list alone"), "{users}");
}

#[test]
fn an_ssh_user_is_searchable_by_the_fingerprint_of_the_key_that_lets_them_in() {
    let page = drawn_seeking(
        &fixture::view(),
        Subject::SshUsers,
        &looking_for("3VaOaGZ8"),
        120,
    );

    assert!(page.contains("deploy"), "{page}");
    assert!(!page.contains("backup"), "{page}");
}
