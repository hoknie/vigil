use vigil_model::Changing;
use vigil_view::{Choice, Field, Form};

use super::editing::Editing;
use crate::ui::{Screen, Spot};

fn editing() -> Editing {
    Editing::open(
        Form::new("EDIT THE ACCOUNT deploy")
            .with(Field::fixed("name", "name", "deploy"))
            .with(Field::text("shell", "shell", "/bin/bash"))
            .with(Field::switch("locked", "locked", false))
            .with(Field::choices(
                "groups",
                "groups",
                vec![Choice::of("wheel", true), Choice::of("docker", false)],
            )),
        Screen::FINDINGS,
        0,
        Some("account|deploy".into()),
        Changing::Update,
    )
}

#[test]
fn a_form_opens_on_its_first_field_a_person_can_change_and_not_on_the_name_it_is_about() {
    assert_eq!(editing().spot(), Spot::Field(1));
}

#[test]
fn the_arrows_walk_from_back_through_the_fields_to_save_and_cancel_and_stop_at_both_ends() {
    let mut editing = editing();

    editing.previous();
    assert_eq!(editing.spot(), Spot::Back);
    editing.previous();
    assert_eq!(
        editing.spot(),
        Spot::Back,
        "the way out at the top does not wrap round to the bottom of the form"
    );

    for wanted in [
        Spot::Field(1),
        Spot::Field(2),
        Spot::Field(3),
        Spot::Save,
        Spot::Cancel,
        Spot::Cancel,
    ] {
        editing.next();
        assert_eq!(editing.spot(), wanted);
    }
}

#[test]
fn a_fixed_field_is_passed_over_because_nothing_in_it_can_be_changed() {
    let mut editing = editing();
    editing.previous();
    editing.next();

    assert_ne!(editing.spot(), Spot::Field(0));
}

#[test]
fn typing_and_erasing_change_the_text_field_the_focus_is_on_and_nothing_else() {
    let mut editing = editing();
    editing.erase();
    editing.erase();
    editing.erase();
    editing.erase();
    for character in "sh".chars() {
        editing.type_character(character);
    }

    assert_eq!(editing.form().text("shell"), Some("/bin/sh"));
    assert!(editing.form().changed("shell"));
    assert!(!editing.form().changed("locked"));
}

#[test]
fn space_turns_a_switch_and_the_choice_under_the_cursor_and_the_side_arrows_walk_choices() {
    let mut editing = editing();
    editing.next();
    editing.toggle();
    assert_eq!(editing.form().switch("locked"), Some(true));

    editing.next();
    editing.sideways(1);
    editing.sideways(1);
    assert_eq!(
        editing.choice(),
        1,
        "the last choice is where the arrow stops"
    );
    editing.toggle();
    assert_eq!(
        editing.form().chosen("groups"),
        Some(vec!["wheel", "docker"])
    );
}

#[test]
fn the_side_arrows_walk_between_save_and_cancel() {
    let mut editing = editing();
    for _ in 0..3 {
        editing.next();
    }
    assert_eq!(editing.spot(), Spot::Save);

    editing.sideways(1);
    assert_eq!(editing.spot(), Spot::Cancel);
    editing.sideways(-1);
    assert_eq!(editing.spot(), Spot::Save);
}

#[test]
fn a_letter_typed_on_a_button_types_nothing() {
    let mut editing = editing();
    editing.previous();
    editing.type_character('x');

    assert_eq!(editing.form().text("shell"), Some("/bin/bash"));
}

#[test]
fn what_went_wrong_stays_on_the_form_beside_what_was_typed() {
    let mut editing = editing();
    editing.type_character('x');
    editing.said("usermod refused");

    assert_eq!(editing.trouble(), Some("usermod refused"));
    assert_eq!(editing.form().text("shell"), Some("/bin/bashx"));
}
