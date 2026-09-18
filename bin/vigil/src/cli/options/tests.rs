use clap::{CommandFactory, Parser};

use super::cli::Cli;
use super::command::Command;
use super::console::{Console, DEFAULT_SOCKET};
use super::moved::{CAPTURE_MOVED, CONFIGURE_LIVES_IN_THE_DAEMON, UI_MOVED, the_old_shape};
use super::opening::Opening;
use super::ui::Ui;
use crate::ui::Screen;
use crate::ui::fixture::screen;

fn parse(line: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("vigil").chain(line.iter().copied()))
}

fn ui(line: &[&str]) -> Ui {
    match parse(line).expect("parses").command {
        Command::Ui(ui) => ui,
        other => panic!("{other:?}"),
    }
}

fn capture(line: &[&str]) -> Console {
    match parse(line).expect("parses").command {
        Command::Capture(console) => console,
        other => panic!("{other:?}"),
    }
}

fn plain(screen: Screen) -> Opening {
    Opening {
        screen,
        difference: false,
    }
}

#[test]
fn opening_the_console_is_a_subcommand_with_the_flags_it_always_had() {
    assert_eq!(
        ui(&["ui"]).console,
        Console {
            socket: DEFAULT_SOCKET.to_string(),
            config: None,
            screen: None,
        }
    );

    assert_eq!(
        ui(&["ui", "--screen", "findings", "--socket", "/tmp/v.sock"]).console,
        Console {
            socket: "/tmp/v.sock".to_string(),
            config: None,
            screen: Some(plain(Screen::FINDINGS)),
        }
    );
}

#[test]
fn the_console_opens_on_the_main_screen_and_a_script_still_gets_the_summary() {
    assert_eq!(
        ui(&["ui"]).console.opening(Screen::HOME),
        plain(Screen::HOME),
        "the main screen is the only door, so it is the one that opens"
    );
    assert_eq!(
        capture(&["capture"]).opening(Screen::SUMMARY),
        plain(Screen::SUMMARY),
        "a watch loop printing `vigil capture` prints the page it always printed"
    );
    assert_eq!(
        ui(&["ui", "--screen", "home"])
            .console
            .opening(Screen::SUMMARY),
        plain(Screen::HOME),
        "and what was asked for wins over either default"
    );
}

#[test]
fn the_sections_a_script_could_already_ask_for_still_mean_what_they_meant() {
    for (asked, expected) in [
        ("summary", Screen::SUMMARY),
        ("ports", screen("network")),
        ("accounts", screen("accounts")),
        ("findings", Screen::FINDINGS),
        ("programs", screen("programs")),
        ("startup", screen("startup")),
        ("home", Screen::HOME),
    ] {
        assert_eq!(
            capture(&["capture", "--screen", asked]).opening(Screen::SUMMARY),
            plain(expected),
            "--screen {asked}"
        );
    }
}

#[test]
fn printing_a_page_is_a_command_of_its_own_and_takes_the_same_two_flags() {
    assert_eq!(
        capture(&["capture"]),
        Console {
            socket: DEFAULT_SOCKET.to_string(),
            config: None,
            screen: None,
        }
    );

    assert_eq!(
        capture(&["capture", "--screen", "accounts", "--socket", "/tmp/v.sock"]),
        Console {
            socket: "/tmp/v.sock".to_string(),
            config: None,
            screen: Some(plain(screen("accounts"))),
        }
    );
}

#[test]
fn the_console_has_no_mode_that_prints_a_page_any_more() {
    assert!(!ui(&["ui"]).once);
    assert!(
        ui(&["ui", "--once"]).once,
        "it still parses, so it can be answered by name"
    );
    assert!(
        !Cli::command()
            .get_subcommands()
            .find(|command| command.get_name() == "ui")
            .expect("ui is a subcommand")
            .get_arguments()
            .any(|argument| argument.get_id() == "once" && !argument.is_hide_set()),
        "`--once` is a tombstone for a script, and must not be offered in the help"
    );
    assert!(CAPTURE_MOVED.contains("vigil capture"));
}

#[test]
fn a_bare_vigil_asks_for_a_command_rather_than_opening_the_console() {
    let error = parse(&[]).expect_err("says what it takes");

    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    );
}

#[test]
fn the_version_is_asked_for_the_way_every_other_program_is_asked_for_it() {
    for spelling in [
        ["-v"].as_slice(),
        ["-V"].as_slice(),
        ["--version"].as_slice(),
    ] {
        let error = parse(spelling).expect_err("a version is printed, not parsed");

        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::DisplayVersion,
            "vigil {spelling:?} did not print the version: {error}"
        );
        assert!(
            error.to_string().contains(env!("CARGO_PKG_VERSION")),
            "vigil {spelling:?}: {error}"
        );
    }
}

#[test]
fn help_is_asked_for_by_its_short_name_too() {
    for spelling in [["-h"].as_slice(), ["--help"].as_slice()] {
        let error = parse(spelling).expect_err("help is printed, not parsed");

        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::DisplayHelp,
            "vigil {spelling:?}: {error}"
        );
    }
}

#[test]
fn the_name_of_a_screen_that_became_a_panel_still_works_and_opens_the_panel() {
    assert_eq!(
        ui(&["ui", "--screen", "difference"]).console.screen,
        Some(Opening {
            screen: Screen::FINDINGS,
            difference: true
        })
    );
    assert_eq!(
        capture(&["capture", "--screen", "difference"]).screen,
        Some(Opening {
            screen: Screen::FINDINGS,
            difference: true
        })
    );
}

#[test]
fn a_screen_that_does_not_exist_says_which_ones_do() {
    for line in [
        ["ui", "--screen", "resources"],
        ["capture", "--screen", "resources"],
    ] {
        let error = parse(&line).expect_err("must not be accepted");

        let said = error.to_string();
        assert!(said.contains("resources"), "{said}");
        assert!(said.contains("summary"), "{said}");
        assert!(said.contains("programs"), "{said}");
        assert!(said.contains("startup"), "{said}");
        assert!(said.contains("home"), "{said}");
        assert!(said.contains("difference"), "{said}");
    }
}

#[test]
fn an_unknown_argument_is_refused_by_name_rather_than_ignored() {
    let error = parse(&["ui", "--follow"]).expect_err("must not be ignored");

    assert!(error.to_string().contains("--follow"), "{error}");
}

#[test]
fn a_flag_that_needs_a_value_and_has_none_says_so() {
    assert!(parse(&["ui", "--socket"]).is_err());
    assert!(parse(&["ui", "--screen"]).is_err());
    assert!(parse(&["capture", "--socket"]).is_err());
    assert!(parse(&["capture", "--screen"]).is_err());
}

#[test]
fn configure_is_a_command_this_binary_knows_the_name_of_and_can_point_at() {
    assert!(matches!(
        parse(&["configure"]).expect("known").command,
        Command::Configure
    ));
    assert!(CONFIGURE_LIVES_IN_THE_DAEMON.contains("vigild configure"));
    assert!(CONFIGURE_LIVES_IN_THE_DAEMON.contains("links no collectors"));
}

#[test]
fn the_old_shape_is_recognised_so_a_script_is_told_where_what_it_asked_for_went() {
    let moved = |line: &[&str]| {
        the_old_shape(
            &std::iter::once("vigil")
                .chain(line.iter().copied())
                .map(str::to_string)
                .collect::<Vec<_>>(),
        )
    };

    assert_eq!(moved(&["--socket", "/tmp/v.sock"]), Some(UI_MOVED));
    assert_eq!(moved(&["--screen", "ports"]), Some(UI_MOVED));
    assert_eq!(moved(&["--screen=ports"]), Some(UI_MOVED));

    assert_eq!(moved(&["--once"]), Some(CAPTURE_MOVED));
    assert_eq!(moved(&["--once", "--screen", "ports"]), Some(CAPTURE_MOVED));
    assert_eq!(moved(&["--screen", "ports", "--once"]), Some(CAPTURE_MOVED));

    assert_eq!(moved(&["ui", "--socket", "/tmp/v.sock"]), None);
    assert_eq!(moved(&["capture"]), None);
    assert_eq!(moved(&[]), None, "a bare vigil asks for a command");
    assert_eq!(moved(&["configure"]), None);
}

#[test]
fn asking_this_program_what_it_is_is_never_read_as_the_shape_it_used_to_have() {
    for asked in ["-v", "-V", "--version", "-h", "--help"] {
        assert_eq!(
            the_old_shape(&["vigil".to_string(), asked.to_string()]),
            None,
            "vigil {asked} was answered with a note about a move"
        );
    }
}
