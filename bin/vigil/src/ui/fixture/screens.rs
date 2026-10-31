use super::super::{Reading, Screen, Status};
use super::{agent, host, readings};

pub fn screen(named: &str) -> Screen {
    Screen::parse(named)
        .unwrap_or_else(|| panic!("{named} is not a screen this build of the console draws"))
}

pub fn of_a_reading_with_no_screen() -> (Status, Reading) {
    let mut status = Status {
        host: host::host(),
        agent: agent::agent(),
        sent_at: "2026-09-09T09:00:01.000Z".into(),
    };
    status
        .agent
        .collectors
        .retain(|collector| crate::ui::Screen::showing(&collector.name).is_some());
    status
        .agent
        .collectors
        .push(agent::collector("kernel", 60, 2, 0));

    (status, Reading::Taken(readings::with_no_screen()))
}
