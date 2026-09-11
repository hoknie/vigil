use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use crate::ui::Subject;

pub(super) fn header(subject: Subject, wide: bool) -> TableRow<'static> {
    let mut names = match subject {
        Subject::Users => vec!["ACCOUNT", "UID", "PASSWORD", "SHELL", "ROOT BY"],
        Subject::Groups => vec!["GROUP", "GID", "GRANTS", "MEMBERS"],
        Subject::Sudo => vec!["WHO", "MAY RUN", "PASSWORD", "REACHES"],
        Subject::Keys => vec!["ACCOUNT", "ALGORITHM", "FINGERPRINT", "USE"],
        Subject::SshUsers => vec!["ACCOUNT", "UID", "KEYS", "SHELL"],
        Subject::LoggedIn => vec!["ACCOUNT", "LINE", "FROM", "WHAT", "SEEN BY"],
        Subject::Other => vec!["KEY", "WHAT THIS CONSOLE CAN SAY"],
    };
    if let Some(extra) = wide.then(|| where_from(subject)).flatten() {
        names.push(extra);
    }
    TableRow::new(names)
}

pub(super) fn where_from(subject: Subject) -> Option<&'static str> {
    match subject {
        Subject::Users => Some("HOME"),
        Subject::Sudo => Some("FROM THE FILE"),
        Subject::Keys | Subject::SshUsers => Some("KEY FILE"),
        Subject::Other => Some("KIND"),
        Subject::LoggedIn => Some("PID"),
        Subject::Groups => None,
    }
}

pub(super) fn widths(subject: Subject, wide: bool) -> Vec<Constraint> {
    let mut widths = match subject {
        Subject::Users => vec![
            Constraint::Min(12),
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Fill(2),
            Constraint::Fill(3),
        ],
        Subject::Groups => vec![
            Constraint::Min(12),
            Constraint::Length(5),
            Constraint::Fill(3),
            Constraint::Fill(2),
        ],
        Subject::Sudo => vec![
            Constraint::Min(12),
            Constraint::Length(13),
            Constraint::Length(12),
            Constraint::Fill(2),
        ],
        Subject::Keys => vec![
            Constraint::Min(10),
            Constraint::Length(12),
            Constraint::Fill(5),
            Constraint::Length(12),
        ],
        Subject::SshUsers => vec![
            Constraint::Min(12),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Fill(2),
        ],
        Subject::LoggedIn => vec![
            Constraint::Min(10),
            Constraint::Length(9),
            Constraint::Fill(2),
            Constraint::Fill(2),
            Constraint::Length(13),
        ],
        Subject::Other => vec![Constraint::Min(20), Constraint::Fill(2)],
    };
    if wide && where_from(subject).is_some() {
        widths.push(Constraint::Fill(2));
    }
    widths
}
