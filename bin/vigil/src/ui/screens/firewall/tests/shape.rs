use super::harness::{drawn, drawn_with, holding, looking_for, without_colour};
use crate::ui::{Audience, Look, fixture};

#[test]
fn the_section_fits_the_narrowest_terminal_this_console_is_read_on() {
    for width in [80u16, 120, 200] {
        for page in [
            drawn(&fixture::view(), width),
            drawn(
                &holding(fixture::firewall::held_by_the_old_backend()),
                width,
            ),
            drawn(&holding(fixture::firewall::filtering_nothing()), width),
        ] {
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
            }
        }
    }
}

#[test]
fn a_policy_of_drop_and_a_policy_of_accept_are_told_apart_without_colour() {
    let page = without_colour(&fixture::view(), 80);

    let dropping: Vec<&str> = page
        .lines()
        .filter(|line| line.contains("inet filter · input"))
        .collect();
    let accepting: Vec<&str> = page
        .lines()
        .filter(|line| line.contains("inet filter · output"))
        .collect();

    assert_eq!(dropping.len(), 1, "{page}");
    assert_eq!(accepting.len(), 1, "{page}");
    assert!(
        dropping[0].contains("drop") && !dropping[0].contains("accept"),
        "the policy is a word in a column of its own, because a colour is nothing on a \
         monochrome terminal: {}",
        dropping[0]
    );
    assert!(accepting[0].contains("accept"), "{}", accepting[0]);
    assert!(
        page.contains("POLICY"),
        "and the column says what the word is: {page}"
    );
}

#[test]
fn the_word_that_says_what_happens_to_an_unmatched_packet_is_never_only_a_colour() {
    let coloured = drawn(&fixture::view(), 80);
    let plain = without_colour(&fixture::view(), 80);

    assert_eq!(
        coloured, plain,
        "every fact on this section is a character, so a palette with no colour in it loses \
         nothing at all: a policy carried by a colour is a policy a NO_COLOR reader cannot \
         see"
    );
}

#[test]
fn the_selected_row_is_marked_with_a_character_and_not_only_a_highlight() {
    let page = drawn(&fixture::view(), 80);

    assert!(page.lines().any(|line| line.starts_with(" > ")), "{page}");
}

#[test]
fn the_whole_ruleset_is_read_before_the_tables_and_a_table_before_its_own_chains() {
    let page = drawn(&fixture::view(), 80);
    let rows: Vec<&str> = page
        .lines()
        .filter(|line| {
            line.contains("ruleset ") || line.contains("table ") || line.contains("chain ")
        })
        .collect();

    let at = |needle: &str| {
        rows.iter()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("{needle} is not drawn:\n{page}"))
    };

    assert!(at("nftables 1.0.6") < at("inet filter "), "{page}");
    assert!(at("inet filter ") < at("inet filter · input"), "{page}");
    assert!(
        at("inet filter · output") < at("ip nat "),
        "the chains of a table come under that table, not after every table: {page}"
    );
}

#[test]
fn a_search_narrows_this_list_and_says_so_rather_than_leaving_an_empty_table() {
    let given = looking_for("nat");
    let page = drawn_with(&fixture::view(), &given, 0, 80, fixture::look());

    assert!(page.contains("ip nat"), "{page}");
    assert!(!page.contains("inet filter"), "{page}");
    assert!(page.contains("of 8 row(s)"), "{page}");

    let nothing = looking_for("wireguard");
    let page = drawn_with(&fixture::view(), &nothing, 0, 80, fixture::look());
    assert!(page.contains("matches"), "{page}");
    assert!(page.contains("wireguard"), "{page}");
}

#[test]
fn a_files_page_puts_the_tally_under_the_table_rather_than_at_the_foot_of_a_tall_page() {
    let mut buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 80, 400));
    crate::ui::screens::firewall::render(
        &fixture::view(),
        Look::new(fixture::monochrome(), Audience::Script),
        &crate::ui::screens::firewall::Showing {
            search: &crate::ui::Search::default(),
            cursor: 0,
            arrows: crate::ui::Arrows::Away,
            gone: None,
            sorting: crate::ui::Sorting::default(),
        },
        buffer.area,
        &mut buffer,
    );

    let page = crate::ui::helpers::words::text::to_text(&buffer);
    let lines: Vec<&str> = page.lines().collect();
    assert!(lines.len() < 20, "{} lines of page: {page}", lines.len());
    assert!(
        lines
            .last()
            .expect("a page")
            .contains("row(s) in this reading"),
        "{page}"
    );
}
