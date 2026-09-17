use std::io::{self, Write};

use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::execute;

use super::clicks::ClicksOnly;

pub fn follow(held: bool, wanted: bool) -> io::Result<bool> {
    if held == wanted {
        return Ok(held);
    }
    let mut out = io::stdout();
    match wanted {
        true => execute!(out, ClicksOnly)?,
        false => execute!(out, DisableMouseCapture)?,
    }
    out.flush()?;
    Ok(wanted)
}

pub fn release() {
    let _ = execute!(io::stdout(), DisableMouseCapture);
    let _ = io::stdout().flush();
}

pub fn release_on_panic() {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panicked| {
        release();
        before(panicked);
    }));
}
