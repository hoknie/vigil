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

#[test]
fn a_sessions_detail_says_who_saw_it_and_what_kind_of_session_it_is() {
    let page = drawn(&fixture::view(), "session|deploy|pts/0", 80);

    assert!(page.contains("DEPLOY"), "{page}");
    assert!(page.contains("pts/0"), "{page}");
    assert!(page.contains("10.0.0.5"), "{page}");
    assert!(page.contains("sshd"), "{page}");
    assert!(page.contains("logind, utmp"), "{page}");
    assert!(page.contains("83"), "the logind session number: {page}");
}

#[test]
fn a_session_nobody_is_sitting_at_says_so_instead_of_reading_like_a_login() {
    let page = drawn(&fixture::view(), "session|root|logind:84", 80);

    assert!(page.contains("closing"), "{page}");
    assert!(page.contains("Nobody is at a terminal"), "{page}");
    assert!(
        page.contains("none: this session holds no terminal"),
        "{page}"
    );
}

#[test]
fn a_login_source_says_where_it_is_read_from_and_whether_it_answered() {
    let there = drawn(&fixture::view(), "session-source|logind", 80);
    let missing = drawn(&fixture::view(), "session-source|utmp", 80);

    assert!(there.contains("/run/systemd/sessions"), "{there}");
    assert!(there.contains("LOGIND"), "{there}");

    assert!(missing.contains("/run/utmp"), "{missing}");
    assert!(
        missing.contains("nothing writes a utmp"),
        "an absent source says why, so nobody reads it as 'nobody is logged in': {missing}"
    );
}
