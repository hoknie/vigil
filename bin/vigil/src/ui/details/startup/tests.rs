use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::render;
use crate::ui::helpers::words::text;
use crate::ui::screens::startup::rows;
use crate::ui::{Search, Startup, fixture};

fn drawn(list: Startup, key: &str) -> String {
    let view = fixture::view();
    let search = Search::default();
    let listed = rows(&view, list, &search);
    let row = listed
        .iter()
        .find(|row| row.key == key)
        .unwrap_or_else(|| panic!("no row keyed {key}"));

    let mut buffer = Buffer::empty(Rect::new(0, 0, 160, 60));
    render(Some(row), fixture::look(), 0, buffer.area, &mut buffer);
    text::to_text(&buffer)
}

#[test]
fn a_unit_says_its_path_what_it_runs_and_as_whom() {
    let page = drawn(Startup::Units, "unit|nginx.service");

    assert!(page.contains("/lib/systemd/system/nginx.service"), "{page}");
    assert!(page.contains("/usr/sbin/nginx -g"), "{page}");
    assert!(page.contains("root"), "{page}");
}

#[test]
fn a_cron_job_says_that_the_whole_command_is_part_of_the_key() {
    let page = drawn(
        Startup::Cron,
        "cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant",
    );

    assert!(page.contains("whole command is part of the key"), "{page}");
    assert!(
        page.contains(
            "persistence|cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant"
        ),
        "{page}"
    );
}

#[test]
fn the_preload_file_lists_what_it_forces_into_every_process() {
    let page = drawn(Startup::Files, "preload|/etc/ld.so.preload");

    assert!(page.contains("/usr/local/lib/libjackit.so"), "{page}");
    assert!(page.contains("every process on this host"), "{page}");
}

#[test]
fn a_watched_file_carries_the_whole_digest_the_table_had_to_cut() {
    let page = drawn(Startup::Files, "script|/etc/profile");

    assert!(
        page.contains("9f2c1b3d4e5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c"),
        "{page}"
    );
}

#[test]
fn a_timer_says_when_it_fires_and_what_it_starts() {
    let page = drawn(Startup::Timers, "timer|logrotate.timer");

    assert!(page.contains("daily"), "{page}");
    assert!(page.contains("logrotate.service"), "{page}");
}
