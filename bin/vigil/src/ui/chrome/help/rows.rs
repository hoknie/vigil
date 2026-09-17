pub(super) fn rows() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "",
            "EVERY RUNG: the arrows move what carries \u{25b8}; \u{2192} in, \u{2190} out",
        ),
        ("1 - 9", "open one; the numbers are on the main screen"),
        ("", "IN A LIST OR A REPORT"),
        (
            "j / k \u{2191} \u{2193}, g / G",
            "a row; PgUp / PgDn a screenful; Home / End the ends",
        ),
        (
            "\u{2192} or Enter",
            "the detail of the row (not on the summary)",
        ),
        ("/", "search: every value the agent read about a row"),
        ("o", "the object a finding is about; Esc comes back to it"),
        (
            "s / f, shift ↑↓",
            "sort/narrow; pick x, a all, d silence them, u undo",
        ),
        (
            "t T u U X, a",
            "show / hide kinds of socket, or all (ports)",
        ),
        (
            "t / d",
            "units as a tree (startup); a section's words (main)",
        ),
        (
            "",
            "SOCKETS, PROGRAMS, ACCOUNTS · n new, e edit on a form, D delete",
        ),
        (
            "\u{2192} \u{2192} \u{2192}",
            "a program opens \u{b7} the panel \u{b7} the arrows move in",
        ),
        (
            "x / M",
            "mark the row (a program takes all of it) / unmark all",
        ),
        (
            "K / U / S",
            "close or stop it \u{b7} U: units, timers, cron / silence",
        ),
        (
            "",
            "THE LISTS OF A SECTION: \u{2190} \u{2192} along them, \u{2193} into one",
        ),
        (
            "",
            "  ports: sockets \u{b7} by program   \u{b7}   system: the host \u{b7} watched files",
        ),
        (
            "",
            "  accounts: users \u{b7} groups \u{b7} sudo \u{b7} keys \u{b7} ssh users \u{b7} logged in",
        ),
        (
            "",
            "  programs: running \u{b7} launches (f: its person or its program)",
        ),
        (
            "",
            "  startup: units \u{b7} timers \u{b7} cron \u{b7} modules \u{b7} files",
        ),
        (
            "\u{2190} or Esc",
            "a search, the detail, the panel, a rung, the main screen",
        ),
        (
            "H / P / r / ? / q",
            "the runs of a launch / the path / ask / list / leave",
        ),
        (
            "m, wheel, Shift",
            "mouse off / on \u{b7} wheel scrolls \u{b7} Shift (Option) selects",
        ),
    ]
}
