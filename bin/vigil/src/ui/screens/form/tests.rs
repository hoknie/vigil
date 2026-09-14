use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::Changing;
use vigil_view::{Choice, Field, Form};

use super::render;
use crate::ui::helpers::words::text;
use crate::ui::{Editing, Screen, fixture};

fn editing() -> Editing {
    Editing::open(
        Form::new("EDIT THE ACCOUNT deploy")
            .saying("The agent runs usermod on this host when this is saved.")
            .with(Field::fixed("name", "name", "deploy"))
            .with(Field::text("shell", "shell", "/bin/bash"))
            .with(
                Field::text("comment", "comment", "")
                    .hinted("not in the reading; left blank it stays as it is"),
            )
            .with(Field::switch("locked", "locked", false))
            .with(Field::choices(
                "groups",
                "groups",
                vec![
                    Choice::of("wheel", true),
                    Choice::of("docker", false),
                    Choice::of("users", false),
                    Choice::of("adm", false),
                ],
            )),
        Screen::FINDINGS,
        0,
        Some("account|deploy".into()),
        Changing::Update,
    )
}

fn drawn(editing: &Editing, width: u16, height: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    render(editing, fixture::look(), buffer.area, &mut buffer);
    text::to_text(&buffer)
}

#[test]
fn the_way_back_is_a_button_at_the_top_and_save_and_cancel_are_buttons_at_the_bottom() {
    let page = drawn(&editing(), 80, 30);
    let lines: Vec<&str> = page.lines().collect();

    assert!(lines[0].contains("[ \u{2190} Back ]"), "{page}");
    let buttons = lines
        .iter()
        .position(|line| line.contains("[ Save ]") && line.contains("[ Cancel ]"))
        .expect("save and cancel are drawn");
    let last_field = lines
        .iter()
        .position(|line| line.contains("groups"))
        .expect("the last field is drawn");
    assert!(buttons > last_field, "{page}");
}

#[test]
fn every_field_is_drawn_with_its_label_and_its_value_in_the_shape_it_takes() {
    let page = drawn(&editing(), 80, 30);

    assert!(page.contains("EDIT THE ACCOUNT deploy"), "{page}");
    assert!(page.contains("usermod"), "{page}");
    assert!(page.contains("deploy"), "{page}");
    assert!(page.contains("[ /bin/bash\u{2582} ]"), "{page}");
    assert!(page.contains("[ ] no"), "{page}");
    assert!(page.contains("[x] wheel"), "{page}");
    assert!(page.contains("[ ] docker"), "{page}");
    assert!(page.contains("left blank it stays"), "{page}");
}

#[test]
fn where_the_focus_is_reads_without_colour() {
    let mut editing = editing();
    let on_shell = drawn(&editing, 80, 30);
    let shell_line = on_shell
        .lines()
        .find(|line| line.contains("shell"))
        .expect("drawn");
    assert!(shell_line.contains('\u{25b8}'), "{on_shell}");
    assert!(
        !on_shell
            .lines()
            .find(|line| line.contains("locked"))
            .expect("drawn")
            .contains('\u{25b8}'),
        "{on_shell}"
    );

    for _ in 0..10 {
        editing.next();
    }
    let on_cancel = drawn(&editing, 80, 30);
    assert!(on_cancel.contains("\u{25b8}[ Cancel ]"), "{on_cancel}");
    assert!(
        !on_cancel.contains("[ /bin/bash\u{2582} ]"),
        "the caret belongs to the field being typed in: {on_cancel}"
    );
}

#[test]
fn it_fits_eighty_and_forty_columns_and_never_runs_off_the_side() {
    for width in [40u16, 80] {
        let page = drawn(&editing(), width, 40);
        for line in page.lines() {
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
        assert!(page.contains("[ Save ]"), "{width}: {page}");
        assert!(page.contains("adm"), "{width}: the choices wrap: {page}");
    }
}

#[test]
fn a_form_taller_than_the_screen_scrolls_to_keep_the_focus_in_sight() {
    let mut editing = editing();
    for _ in 0..10 {
        editing.next();
    }

    let page = drawn(&editing, 80, 6);

    assert!(page.contains("[ Cancel ]"), "{page}");
}

#[test]
fn what_the_agent_refused_is_written_above_the_buttons() {
    let mut editing = editing();
    editing.said("usermod: user deploy is currently used by process 4242");

    let page = drawn(&editing, 80, 30);

    assert!(page.contains("currently used by process"), "{page}");
}
