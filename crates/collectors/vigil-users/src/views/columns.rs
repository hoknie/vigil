use vigil_view::{Column, Width};

use crate::types::Subject;

pub(super) fn columns(subject: Subject, wide: bool) -> Vec<Column> {
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

    names
        .into_iter()
        .zip(widths(subject, wide))
        .map(|(header, width)| Column::new(header, width))
        .collect()
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

fn widths(subject: Subject, wide: bool) -> Vec<Width> {
    let mut widths = match subject {
        Subject::Users => vec![
            Width::Least(12),
            Width::Fixed(5),
            Width::Fixed(8),
            Width::Share(2),
            Width::Share(3),
        ],
        Subject::Groups => vec![
            Width::Least(12),
            Width::Fixed(5),
            Width::Share(3),
            Width::Share(2),
        ],
        Subject::Sudo => vec![
            Width::Least(12),
            Width::Fixed(13),
            Width::Fixed(12),
            Width::Share(2),
        ],
        Subject::Keys => vec![
            Width::Least(10),
            Width::Fixed(12),
            Width::Share(5),
            Width::Fixed(12),
        ],
        Subject::SshUsers => vec![
            Width::Least(12),
            Width::Fixed(5),
            Width::Fixed(10),
            Width::Share(2),
        ],
        Subject::LoggedIn => vec![
            Width::Least(10),
            Width::Fixed(9),
            Width::Share(2),
            Width::Share(2),
            Width::Fixed(13),
        ],
        Subject::Other => vec![Width::Least(20), Width::Share(2)],
    };
    if wide && where_from(subject).is_some() {
        widths.push(Width::Share(2));
    }
    widths
}
