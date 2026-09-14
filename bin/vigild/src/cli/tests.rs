use clap::Parser;

use super::options::{Cli, Collector, Command, Configure, Switch};
use crate::wizard::DEFAULT_PATH;

fn collector(line: &[&str]) -> Collector {
    match parse(line).expect("parses").command {
        Some(Command::Collector(asked)) => asked,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_name_of_the_collector_is_an_argument_and_not_half_the_name_of_the_command() {
    let asked = collector(&["collector", "firewall", "enable"]);

    assert_eq!(asked.name, "firewall");
    assert!(matches!(asked.doing, Switch::Enable(_)));
    assert_eq!(asked.switching().config, DEFAULT_PATH);
    assert!(!asked.switching().dry_run);
}

#[test]
fn both_ways_round_are_spelled_the_same_and_take_the_same_flags() {
    for (line, dry) in [
        (vec!["collector", "users", "enable", "-d"], true),
        (vec!["collector", "users", "disable", "--dry-run"], true),
        (vec!["collector", "users", "disable"], false),
    ] {
        let asked = collector(&line);
        assert_eq!(asked.name, "users");
        assert_eq!(asked.switching().dry_run, dry, "{line:?}");
    }
}

#[test]
fn the_file_it_edits_can_be_named_so_a_check_need_not_write_the_real_one() {
    let asked = collector(&["collector", "ports", "enable", "--config", "/tmp/v.yaml"]);

    assert_eq!(asked.switching().config, "/tmp/v.yaml");
    assert_eq!(asked.options().path, "/tmp/v.yaml");
    assert_eq!(asked.options().name, "ports");
}

#[test]
fn a_word_that_is_neither_of_the_two_is_refused_rather_than_read_as_a_name() {
    assert!(parse(&["collector", "firewall", "enabel"]).is_err());
    assert!(parse(&["collector", "firewall"]).is_err());
    assert!(parse(&["collector"]).is_err());
}

fn parse(line: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("vigild").chain(line.iter().copied()))
}

fn configure(line: &[&str]) -> Configure {
    match parse(line).expect("parses").command {
        Some(Command::Configure(options)) => options,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_version_is_asked_for_the_way_every_other_program_is_asked_for_it() {
    for spelling in ["-v", "-V", "--version"] {
        let error = parse(&[spelling]).expect_err("a version is printed, not parsed");

        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::DisplayVersion,
            "vigild {spelling} did not print the version: {error}"
        );
        assert!(
            error.to_string().contains(env!("CARGO_PKG_VERSION")),
            "vigild {spelling}: {error}"
        );
    }
}

#[test]
fn a_path_on_its_own_is_the_configuration_to_watch_with() {
    let parsed = parse(&["/etc/vigil/vigil.yaml"]).expect("parses");

    assert_eq!(parsed.config.as_deref(), Some("/etc/vigil/vigil.yaml"));
    assert!(parsed.command.is_none());
}

#[test]
fn configure_with_nothing_else_writes_where_the_package_installs() {
    assert_eq!(
        configure(&["configure"]),
        Configure {
            path: DEFAULT_PATH.to_string(),
            force: false,
            dry_run: false,
        }
    );
}

#[test]
fn both_flags_have_a_short_form_and_a_long_one_and_a_path_can_be_named() {
    assert_eq!(
        configure(&["configure", "-f", "-d", "/tmp/v.yaml"]),
        Configure {
            path: "/tmp/v.yaml".to_string(),
            force: true,
            dry_run: true,
        }
    );
    assert_eq!(
        configure(&["configure", "--force", "--dry-run"]),
        Configure {
            path: DEFAULT_PATH.to_string(),
            force: true,
            dry_run: true,
        }
    );
}

#[test]
fn a_flag_in_the_place_of_a_path_is_refused_rather_than_read_as_a_file_name() {
    let error = parse(&["--dry-run"]).expect_err("refused");
    assert!(error.to_string().contains("--dry-run"), "{error}");

    let error = parse(&["configure", "--forse"]).expect_err("refused");
    assert!(error.to_string().contains("--forse"), "{error}");
}

#[test]
fn a_second_path_is_refused_rather_than_silently_ignored() {
    assert!(parse(&["a.yaml", "b.yaml"]).is_err());
    assert!(parse(&["configure", "a.yaml", "b.yaml"]).is_err());
}

#[test]
fn nothing_on_the_command_line_is_the_help_and_never_a_daemon_watching_a_default_file() {
    let error = parse(&[]).expect_err("nothing to run, so nothing is parsed");

    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    );

    let shown = error.to_string();
    assert!(shown.contains("configure"), "{shown}");
    assert!(shown.contains("collector"), "{shown}");
    assert!(shown.contains("CONFIG"), "{shown}");
}

#[test]
fn the_help_says_what_the_two_things_it_does_are() {
    let help = parse(&["--help"])
        .expect_err("help is not a parse")
        .to_string();

    assert!(help.contains("configure"), "{help}");
    assert!(help.contains("CONFIG"), "{help}");
    assert!(help.contains("vigil ui"), "the console is named: {help}");
}
