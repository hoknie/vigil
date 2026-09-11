use super::harness::drawn;
use crate::ui::screens::home::rows;
use crate::ui::{Screen, fixture};

fn numbers_drawn(page: &str) -> Vec<(String, String)> {
    page.lines()
        .filter_map(|line| {
            let letters: Vec<char> = line.chars().collect();
            let number = *letters.get(5)?;
            if !number.is_ascii_digit() || *letters.get(6)? != ' ' {
                return None;
            }
            let name = line.chars().skip(7).collect::<String>();
            Some((
                number.to_string(),
                name.split_whitespace().next()?.to_string(),
            ))
        })
        .collect()
}

#[test]
fn the_number_drawn_on_a_row_is_the_number_that_opens_it() {
    let view = fixture::view();
    let page = drawn(&view, 120, 30);

    let drawn_pairs = numbers_drawn(&page);

    assert!(!drawn_pairs.is_empty(), "no numbers on the page: {page}");
    for (number, name) in drawn_pairs {
        let screen = Screen::parse(&name)
            .unwrap_or_else(|| panic!("{name} is drawn as a section and is not one: {page}"));
        assert_eq!(
            screen.digit().map(|it| it.to_string()),
            Some(number.clone()),
            "the row for {name} is drawn with {number} and {number} opens something else"
        );
    }
}

#[test]
fn a_section_past_the_ninth_draws_no_number_and_is_opened_by_the_cursor() {
    let view = fixture::view();

    for row in rows(&view) {
        match row.opens.and_then(|screen| screen.digit()) {
            Some(number) => assert_eq!(row.number, Some(number)),
            None => assert_eq!(
                row.number, None,
                "a row with no working number must leave the field empty: a dash there reads \
                 as a value"
            ),
        }
    }
}

#[test]
fn the_range_named_in_the_footer_is_the_range_that_opens_something() {
    let page = drawn(&fixture::view(), 80, 30);

    let footer = page
        .lines()
        .rev()
        .find(|line| line.contains("section(s)"))
        .expect("a footer");
    assert!(
        footer.contains(&format!("1 - {}", Screen::numbered())),
        "{footer}"
    );
}
