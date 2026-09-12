use super::row::Row;
use crate::ui::Screen;

pub(super) fn tally(rows: &[Row], width: u16) -> String {
    let sections = rows.iter().filter(|row| row.opens.is_some()).count();
    let numbered = Screen::numbered();
    let off = rows
        .iter()
        .filter(|row| row.standing.state == "off")
        .count();
    let unwell = rows.iter().filter(|row| row.standing.unwell).count();

    let mut parts = vec![format!("{sections} section(s)")];
    parts.push(match numbered {
        0 => "nothing here opens with a number".to_string(),
        1 => "1 opens the first from anywhere".to_string(),
        last => format!("1 - {last} open one from anywhere"),
    });
    if let Some(named) = without_a_number() {
        parts.push(format!("{named} opens with Enter on its row"));
    }
    if rows.len() > sections {
        parts.push(format!(
            "{} reading(s) this console has no section for",
            rows.len() - sections
        ));
    }
    if unwell > 0 {
        parts.push(format!("{unwell} reading(s) less than asked for"));
    }
    if off > 0 {
        parts.push(format!("{off} switched off"));
    }

    let room = (width as usize).saturating_sub(1);
    let mut line = String::new();
    for part in parts {
        let next = match line.is_empty() {
            true => part,
            false => format!("{line} · {part}"),
        };
        if next.chars().count() > room {
            break;
        }
        line = next;
    }
    format!(" {line}")
}

fn without_a_number() -> Option<String> {
    let named: Vec<&str> = Screen::unnumbered()
        .iter()
        .map(|screen| screen.name())
        .collect();

    match named.is_empty() {
        true => None,
        false => Some(named.join(" and ")),
    }
}
