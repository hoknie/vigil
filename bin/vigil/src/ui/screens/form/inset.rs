use ratatui::text::Span;

pub const LEFT: &str = "\u{2595}";

pub const RIGHT: &str = "\u{258f}";

pub const OPENS: &str = "\u{25be}";

pub const EDGES: usize = 4;

pub fn visible(value: &str, scroll: usize, width: usize) -> String {
    let mut skipped = 0;
    let mut used = 0;
    let mut shown = String::new();
    for character in value.chars() {
        let wide = Span::raw(character.to_string()).width();
        if skipped < scroll {
            skipped += wide;
            continue;
        }
        if used + wide > width {
            break;
        }
        used += wide;
        shown.push(character);
    }
    shown.extend(std::iter::repeat_n(' ', width.saturating_sub(used)));
    shown
}

pub fn summary(chosen: &[&str], width: usize) -> String {
    let said = match chosen.is_empty() {
        true => "none".to_string(),
        false => chosen.join(", "),
    };
    let room = width.saturating_sub(2);
    let count = said.chars().count();
    let fitted = match count > room {
        true => {
            said.chars()
                .take(room.saturating_sub(1))
                .collect::<String>()
                + "\u{2026}"
        }
        false => said + &" ".repeat(room - count),
    };
    format!("{fitted} {OPENS}")
}
