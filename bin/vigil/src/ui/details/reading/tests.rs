use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde_json::json;

use super::{Subject, render};
use crate::ui::fixture;
use crate::ui::helpers::words::text;

fn drawn(subject: Subject<'_>) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    render(Some(subject), fixture::look(), 0, buffer.area, &mut buffer);
    text::to_text(&buffer)
}

fn about<'a>(item: &'a serde_json::Value, means: &[&str]) -> Subject<'a> {
    Subject {
        key: "fs|/var".to_string(),
        kind: "filesystem",
        named: "/var".to_string(),
        means: means.iter().map(|said| (*said).to_string()).collect(),
        item,
    }
}

#[test]
fn every_value_the_agent_recorded_about_the_row_is_named_and_so_is_the_object_key() {
    let item = json!({"mount": "/var", "free_percent_step": 5, "read_only": false});

    let page = drawn(about(&item, &[]));

    assert!(page.contains("FILESYSTEM  /var"), "{page}");
    assert!(page.contains("mount"), "{page}");
    assert!(page.contains("free percent step"), "{page}");
    assert!(page.contains("object"), "{page}");
    assert!(page.contains("fs|/var"), "{page}");
}

#[test]
fn a_row_with_nothing_of_its_own_to_say_draws_no_block_saying_so() {
    let item = json!({"mount": "/var"});

    let page = drawn(about(&item, &[]));

    assert!(
        !page.contains("WHAT THIS IS"),
        "a heading over a sentence true of every row of every reading tells a reader \
         nothing about the row they are looking at: {page}"
    );
}

#[test]
fn a_row_with_something_of_its_own_to_say_says_it_under_a_heading() {
    let item = json!({"mount": "/var"});

    let page = drawn(about(&item, &["This filesystem is mounted read only."]));

    assert!(page.contains("WHAT THIS IS"), "{page}");
    assert!(page.contains("mounted read only"), "{page}");
}

#[test]
fn a_list_of_values_is_drawn_as_a_list_and_counted_rather_than_run_together() {
    let item = json!({"host_paths": ["/srv/www", "/run/docker.sock"]});
    let page = drawn(Subject {
        key: "container|3ab1".to_string(),
        kind: "container",
        named: "3ab1".to_string(),
        means: Vec::new(),
        item: &item,
    });

    assert!(page.contains("2 in all"), "{page}");
    assert!(page.contains("· /srv/www"), "{page}");
    assert!(page.contains("· /run/docker.sock"), "{page}");
}

#[test]
fn nothing_runs_off_the_side_at_the_narrowest_this_is_read_at() {
    let item =
        json!({"sha256": "87e95e7564445110057c3e0563e80077ccdab840ac81fccc5e66aaef991bb298"});
    let page = drawn(Subject {
        key: "file|/etc/ssh/sshd_config".to_string(),
        kind: "file",
        named: "/etc/ssh/sshd_config".to_string(),
        means: vec![
            "This file is past the size this agent hashes, so it is watched by its \
             permissions and its owner alone."
                .to_string(),
        ],
        item: &item,
    });

    for line in page.lines() {
        assert!(line.chars().count() <= 80, "{line}");
    }
}
