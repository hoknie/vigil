use std::time::Instant;

use ratatui::crossterm::event::KeyCode;

use crate::ui::app::tests::harness::{app, drawn_at, number, press};
use crate::ui::fixture::screen;
use crate::ui::{Level, Reading as Held};

fn many_launches(copies: usize) -> vigil_model::Snapshot {
    let mut reading = vigil_launches::fixture::launches();
    let read = reading.items.clone();
    for copy in 1..copies {
        for (key, item) in &read {
            reading.items.insert(format!("{key}~{copy:04}"), item.clone());
        }
    }
    reading
}

fn timed(rounds: usize, mut pressed: impl FnMut()) -> f64 {
    let started = Instant::now();
    for _ in 0..rounds {
        pressed();
    }
    started.elapsed().as_secs_f64() * 1000.0 / rounds as f64
}

#[test]
#[ignore]
fn measure_choosing_a_facet_on_thirteen_thousand_launches() {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    let reading = many_launches(2267);
    let rows = reading.items.len();
    app.view.readings.put("launches", Held::Taken(reading));
    press(&mut app, number(screen("programs")));
    drawn_at(&app, 120, 30);
    press(&mut app, KeyCode::Right);
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, 120, 30);
    for _ in 0..3 {
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, 120, 30);

    let arrow = timed(40, || {
        press(&mut app, KeyCode::Down);
        drawn_at(&app, 120, 30);
    });
    press(&mut app, KeyCode::Char('f'));
    drawn_at(&app, 120, 30);
    let walking = timed(40, || {
        press(&mut app, KeyCode::Right);
        drawn_at(&app, 120, 30);
    });
    let mut chosen = 0.0;
    for round in 0..20 {
        press(&mut app, KeyCode::Char('f'));
        drawn_at(&app, 120, 30);
        let wanted = 1 + round % 2;
        while app.chooser.at() != wanted {
            press(&mut app, KeyCode::Right);
        }
        drawn_at(&app, 120, 30);
        chosen += timed(1, || {
            press(&mut app, KeyCode::Enter);
            drawn_at(&app, 120, 30);
        });
        press(&mut app, KeyCode::Char('f'));
        drawn_at(&app, 120, 30);
        while app.chooser.at() != 0 {
            press(&mut app, KeyCode::Right);
        }
        press(&mut app, KeyCode::Enter);
        drawn_at(&app, 120, 30);
    }
    println!(
        "rows {rows}: arrow {arrow:.3} ms, a step through the facets {walking:.3} ms, a facet chosen {:.3} ms",
        chosen / 20.0
    );
}
