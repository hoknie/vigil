use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde_json::json;
use vigil_model::{Evidence, Finding, Severity};

use crate::ui::helpers::finding::diff::{height, render};
use crate::ui::helpers::words::text;
use crate::ui::{Audience, Deed, Look, fixture};

fn drawn(finding: Option<&Finding>) -> String {
    drawn_at(finding, 80)
}

fn drawn_at(finding: Option<&Finding>, width: u16) -> String {
    picking(finding, width, 0)
}

fn picking(finding: Option<&Finding>, width: u16, picked: usize) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 60));
    render(
        finding,
        fixture::look(),
        0,
        picked,
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

#[test]
fn it_shows_what_the_object_was_and_what_it_became() {
    let mut finding = fixture::finding("The owner of :443 changed", Severity::High);
    finding.before = Some(json!({"process": {"exe": "/usr/sbin/nginx"}, "user": "root"}));
    finding.after = Some(json!({"process": {"exe": "/tmp/.x/nc"}, "user": "www-data"}));

    let page = drawn(Some(&finding));

    assert!(page.contains("- process.exe"), "{page}");
    assert!(page.contains("/usr/sbin/nginx"), "{page}");
    assert!(page.contains("+ process.exe"), "{page}");
    assert!(page.contains("/tmp/.x/nc"), "{page}");
}

#[test]
fn what_was_hidden_before_writing_is_named_rather_than_shown_as_empty() {
    let mut finding = fixture::finding("A new listening port", Severity::Critical);
    finding.after = Some(json!({"process": {"cmdline": "mysql -p<hidden>"}}));
    finding.redacted = vec!["/after/process/cmdline".into()];

    let page = drawn(Some(&finding));

    assert!(page.contains("HIDDEN"), "{page}");
    assert!(page.contains("/after/process/cmdline"), "{page}");
}

#[test]
fn evidence_is_on_the_screen_because_it_is_what_makes_a_finding_investigable() {
    let mut finding = fixture::finding("A new listening port", Severity::Critical);
    finding.evidence = vec![Evidence {
        kind: "path".into(),
        value: "/tmp/.x/nc".into(),
    }];

    assert!(drawn(Some(&finding)).contains("/tmp/.x/nc"));
}

#[test]
fn it_spells_the_configuration_entry_that_would_silence_this_finding() {
    let mut finding = fixture::finding("A group gained a member", Severity::High);
    finding.finding_key = "user|group|docker".into();

    let page = drawn(Some(&finding));

    assert!(page.contains("suppressions:"), "{page}");
    assert!(
        page.contains("finding_key: \"user|group|docker\""),
        "{page}"
    );
    assert!(page.contains("reason:"), "{page}");
}

#[test]
fn a_long_object_key_is_shown_whole_rather_than_cut_where_nobody_can_see_the_cut() {
    let mut finding = fixture::finding("A key was added", Severity::High);
    finding.finding_key = format!("user|sshkey|deploy|SHA256:{}", "a".repeat(60));

    let page = drawn(Some(&finding));
    let unbroken: String = page.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        unbroken.contains(&finding.finding_key),
        "the key is not all there: {page}"
    );
    for line in page.lines() {
        assert!(line.chars().count() <= 80, "{line}");
    }
}

#[test]
fn a_long_title_is_wrapped_because_the_title_is_the_point_of_the_screen() {
    let mut finding = fixture::finding(
        "A new SSH key may log in as intruder: ssh-ed25519 \
         SHA256:YpTjnHs8O4g1NgWt/Kcm+JLcyjBkWlA25hV/c7zRKRU person@laptop",
        Severity::High,
    );
    finding.finding_key = "user|sshkey|intruder".into();

    let page = drawn(Some(&finding));

    assert!(
        page.contains("person@laptop"),
        "the end of it survived: {page}"
    );
    for line in page.lines() {
        assert!(line.chars().count() <= 80, "{line}");
    }
}

#[test]
fn the_panel_of_a_finding_offers_the_same_deeds_as_the_bar_over_the_picked_rows() {
    let finding = fixture::finding("A new listening port", Severity::Critical);

    let page = drawn(Some(&finding));

    assert!(page.contains("ACTIONS"), "{page}");
    for deed in Deed::ALL {
        assert!(
            page.contains(deed.alone()),
            "{} is not offered: {page}",
            deed.key()
        );
    }
}

#[test]
fn with_rows_picked_the_panel_says_the_deed_reaches_them_and_not_only_this_one() {
    let finding = fixture::finding("A new listening port", Severity::Critical);

    let page = picking(Some(&finding), 80, 3);

    assert!(page.contains("all 3 of them"), "{page}");
    assert!(
        !page.contains(Deed::Remove.alone()),
        "one row and three rows must not read the same: {page}"
    );
}

#[test]
fn a_page_written_to_a_file_offers_no_key_to_press_on_it() {
    let finding = fixture::finding("A new listening port", Severity::Critical);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 60));
    render(
        Some(&finding),
        Look::new(fixture::monochrome(), Audience::Script),
        0,
        0,
        buffer.area,
        &mut buffer,
    );

    let page = text::to_text(&buffer);
    assert!(
        !page.contains("ACTIONS"),
        "nothing reading a file can press d: {page}"
    );
    assert!(
        page.contains("suppressions:"),
        "and what it can copy is still there: {page}"
    );
}

#[test]
fn with_nothing_selected_it_says_how_to_select_something() {
    assert!(drawn(None).contains("press Enter"));
}

#[test]
fn the_page_is_as_long_as_it_is_so_the_scroll_position_can_be_kept_inside_it() {
    let finding = fixture::finding("A new listening port", Severity::Critical);

    assert!(height(Some(&finding), fixture::look(), 80) > 10);
    assert_eq!(height(None, fixture::look(), 80), 0);
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    let mut finding = fixture::finding("A new listening port", Severity::Critical);
    finding.after = Some(json!({"process": {"cmdline": "a".repeat(300)}}));

    for width in [80u16, 120, 200] {
        for line in drawn_at(Some(&finding), width).lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
    }
}
