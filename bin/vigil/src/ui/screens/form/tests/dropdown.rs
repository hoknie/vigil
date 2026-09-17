use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use vigil_model::Changing;
use vigil_view::{Choice, Field, Form};

use super::drawing::{drawn_with, editing};
use crate::ui::helpers::motion::form::pressed;
use crate::ui::{Aim, Editing, Pressed, Screen, Spot};

fn on_the_groups(mut editing: Editing) -> Editing {
    while !editing.in_a_list() {
        editing.next();
    }
    editing
}

fn press(editing: &mut Editing, code: KeyCode) -> Pressed {
    pressed(editing, code, KeyModifiers::NONE)
}

fn crowded() -> Editing {
    Editing::open(
        Form::new("EDIT THE ACCOUNT deploy")
            .saying("The agent runs usermod on this host when this is saved.")
            .with(Field::fixed("name", "name", "deploy"))
            .with(Field::text("shell", "shell", "/bin/bash"))
            .with(Field::choices(
                "groups",
                "groups",
                (0..40)
                    .map(|at| Choice::of(format!("group{at:02}"), at == 3))
                    .collect(),
            ))
            .with(Field::text("comment", "comment", "")),
        Screen::FINDINGS,
        0,
        Some("account|deploy".into()),
        Changing::Update,
    )
}

#[test]
fn a_closed_list_reads_as_what_is_chosen_and_an_arrow_that_opens_it_without_colour() {
    let mut editing = on_the_groups(editing());
    let (page, cursor, _) = drawn_with(&editing, 80, 30);
    let line = page
        .lines()
        .find(|line| line.contains("groups"))
        .expect("drawn");
    assert!(line.contains("\u{2595} wheel"), "{page}");
    assert!(line.trim_end().ends_with("\u{25be} \u{258f}"), "{page}");
    assert!(
        !page.contains("[ ] docker"),
        "a closed list lists nothing: {page}"
    );
    assert_eq!(cursor, None, "there is nothing to type into here");

    press(&mut editing, KeyCode::Enter);
    press(&mut editing, KeyCode::Char(' '));
    press(&mut editing, KeyCode::Esc);
    let (page, _, _) = drawn_with(&editing, 80, 30);
    assert!(
        page.lines()
            .any(|line| line.contains("groups") && line.contains("\u{2595} none")),
        "nothing chosen is said, not left blank: {page}"
    );
}

#[test]
fn an_open_list_is_drawn_under_its_field_in_a_frame_with_a_mark_on_every_row() {
    let mut editing = on_the_groups(editing());
    press(&mut editing, KeyCode::Enter);

    let (page, _, aims) = drawn_with(&editing, 80, 30);
    let lines: Vec<&str> = page.lines().collect();
    let field = lines
        .iter()
        .position(|line| line.contains("groups"))
        .expect("drawn");

    assert!(
        lines[field + 1].contains('\u{256d}'),
        "the frame opens under the field: {page}"
    );
    assert!(lines[field + 2].contains("\u{25b8} [x] wheel"), "{page}");
    assert!(lines[field + 3].contains("  [ ] docker"), "{page}");
    assert!(
        page.contains("toggles kept"),
        "the footing says Esc keeps them: {page}"
    );
    let rows: Vec<Aim> = aims
        .iter()
        .filter(|(aim, _)| matches!(aim, Aim::Choice(_)))
        .map(|(aim, _)| *aim)
        .collect();
    assert_eq!(
        rows,
        vec![
            Aim::Choice(0),
            Aim::Choice(1),
            Aim::Choice(2),
            Aim::Choice(3)
        ]
    );
}

#[test]
fn space_toggles_letters_narrow_and_esc_closes_the_list_keeping_the_toggles_and_the_form() {
    let mut editing = on_the_groups(editing());
    assert_eq!(press(&mut editing, KeyCode::Char(' ')), Pressed::Nothing);
    assert!(
        editing.dropdown().is_some(),
        "space opens a closed list as Enter does"
    );

    press(&mut editing, KeyCode::Down);
    press(&mut editing, KeyCode::Char(' '));
    for letter in "ad".chars() {
        press(&mut editing, KeyCode::Char(letter));
    }
    let (page, _, _) = drawn_with(&editing, 80, 30);
    assert!(page.contains("narrowed to ad"), "{page}");
    assert!(page.contains("\u{25b8} [ ] adm"), "{page}");
    assert!(
        !page.contains("[x] wheel"),
        "a narrowed list shows only what matches: {page}"
    );
    press(&mut editing, KeyCode::Char(' '));

    assert_eq!(
        press(&mut editing, KeyCode::Esc),
        Pressed::Nothing,
        "Esc in an open list closes the list, not the form"
    );
    assert!(editing.dropdown().is_none());
    assert_eq!(
        editing.form().chosen("groups"),
        Some(vec!["wheel", "docker", "adm"]),
        "every toggle made while it was open is kept"
    );
    assert_eq!(press(&mut editing, KeyCode::Esc), Pressed::Leave);
}

#[test]
fn a_long_list_stays_on_an_eighty_by_twenty_four_screen_scrolls_and_does_not_push_the_form() {
    let mut editing = on_the_groups(crowded());
    let (closed, _, _) = drawn_with(&editing, 80, 22);
    press(&mut editing, KeyCode::Enter);
    for _ in 0..30 {
        press(&mut editing, KeyCode::Down);
    }

    let (page, _, aims) = drawn_with(&editing, 80, 22);
    let screen = Rect::new(0, 0, 80, 22);
    for (aim, area) in &aims {
        assert!(
            area.right() <= screen.right() && area.bottom() <= screen.bottom(),
            "{aim:?} at {area:?}: {page}"
        );
    }
    assert!(
        page.contains("\u{25b8} [ ] group30"),
        "the list follows the cursor: {page}"
    );
    assert!(
        page.contains('\u{2588}'),
        "a scrollbar says there is more: {page}"
    );
    let above = |page: &str| {
        page.lines()
            .take_while(|line| !line.contains("groups"))
            .map(str::to_string)
            .collect::<Vec<String>>()
    };
    assert_eq!(
        above(&closed),
        above(&page),
        "the form under the list did not move"
    );
    assert_eq!(
        aims.iter()
            .find(|(aim, _)| *aim == Aim::Spot(Spot::Field(2)))
            .map(|(_, area)| area.y),
        closed
            .lines()
            .position(|line| line.contains("groups"))
            .map(|at| at as u16)
    );
}

#[test]
fn nothing_matching_what_was_typed_is_said_inside_the_list() {
    let mut editing = on_the_groups(editing());
    press(&mut editing, KeyCode::Enter);
    for letter in "zz".chars() {
        press(&mut editing, KeyCode::Char(letter));
    }

    let (page, _, _) = drawn_with(&editing, 80, 30);

    assert!(page.contains("nothing matches zz"), "{page}");
    press(&mut editing, KeyCode::Char(' '));
    assert_eq!(editing.form().chosen("groups"), Some(vec!["wheel"]));
}
