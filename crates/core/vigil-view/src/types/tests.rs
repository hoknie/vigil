use super::*;

#[test]
fn a_column_the_terminal_has_no_room_for_is_not_offered_to_the_renderer() {
    let columns = vec![
        Column::new("ADDRESS", Width::Least(24)),
        Column::new("COMMAND", Width::Share(2)).shown_from(118),
    ];

    let narrow = fitting(columns.clone(), Room::of(80));
    let wide = fitting(columns, Room::of(120));

    assert_eq!(narrow.len(), 1, "{narrow:?}");
    assert_eq!(wide.len(), 2);
}

#[test]
fn a_row_a_pane_made_up_says_so_and_is_never_looked_for_in_the_reading() {
    let heading = RowKey::of("program|/usr/bin/nc").of_its_own();
    let socket = RowKey::of("tcp|0.0.0.0:443");

    assert!(
        !heading.of_the_reading,
        "a heading a pane groups rows under is not an item the collector wrote, and a \
         finding must never be walked to it"
    );
    assert!(socket.of_the_reading);
}

#[test]
fn a_pane_whose_filter_nobody_has_touched_is_shown_whole_rather_than_shown_empty() {
    let untouched = Showing::default();
    let hiding = Showing::default().hiding(&["udp"]);

    assert!(
        untouched.wants("udp"),
        "what the reader has switched off is what is hidden; a reader who has switched off \
         nothing is shown everything"
    );
    assert!(hiding.wants("tcp"));
    assert!(!hiding.wants("udp"));
    assert!(hiding.narrowed());
}

#[test]
fn a_list_narrowed_to_a_value_names_the_value_and_counts_as_holding_rows_back() {
    let only = [Facet::new("user", "alice")];
    let showing = Showing::default().narrowing(&only);

    assert_eq!(showing.only("user"), Some("alice"));
    assert_eq!(showing.only("program"), None);
    assert!(
        showing.holding_back(),
        "a footer that says 12 launch(es) over a list of three, because nobody typed a \
         search, is a footer that lies about what the reader is looking at"
    );
    assert!(!Showing::default().holding_back());
}

#[test]
fn every_column_is_offered_both_ways_round_and_the_choice_comes_back_the_same() {
    for chosen in 0..7 {
        assert_eq!(Sorting::of(chosen).chosen(), chosen);
    }
    assert!(Sorting::of(0).as_read());
    assert!(!Sorting::of(1).descending);
    assert!(Sorting::of(2).descending);
}

#[test]
fn a_notice_keeps_its_sentences_in_the_order_they_were_said() {
    let notice = Notice::loud("nothing is listening")
        .saying("the daemon answered")
        .saying("and the reading was empty");

    assert!(notice.loud);
    assert_eq!(notice.detail.len(), 2);
    assert_eq!(notice.detail[0], "the daemon answered");
}

#[test]
fn a_pane_says_what_it_offers_and_the_default_is_everything_a_list_can_do() {
    let all = Offers::default();
    let none = Offers::nothing();

    assert!(all.sorting && all.search && all.detail);
    assert!(!none.sorting && !none.search && !none.detail);
    assert!(!Offers::default().sorted(false).sorting);
}
