use clap::{Parser, Subcommand};

use super::style;
use crate::collector;
use crate::wizard::{DEFAULT_PATH, Options};

#[derive(Debug, Parser)]
#[command(
    name = "vigild",
    version,
    disable_version_flag = true,
    styles = style::HELP,
    about = "Watch this host, and report what changes about it.",
    long_about = "\
Watch this host: what is listening, who can log in, what runs at boot, what is running now.
Each reading is compared with the last one, and the difference becomes findings.",
    disable_help_subcommand = true,
    args_conflicts_with_subcommands = true,
    arg_required_else_help = true,
    after_help = "\
The console over the findings is `vigil ui`.",
)]
pub struct Cli {
    #[arg(
        short = 'v',
        short_alias = 'V',
        long = "version",
        action = clap::ArgAction::Version,
        help = "Print the version and leave"
    )]
    pub version: (),

    #[arg(
        value_name = "CONFIG",
        help = "The configuration file to watch this host with",
        long_help = "\
The configuration file to watch this host with.

It is named, never guessed. The shipped systemd unit names it on an installed
host."
    )]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Ask this host what it can be watched with, and write the configuration for it",
        long_about = "\
Ask every collector what it can read on THIS host: the kernel it has, the files it may open,
the privileges it was started with. The configuration names what it found and what it could not.
A collector that cannot read here is written out switched off, with the reason beside it."
    )]
    Configure(Configure),

    #[command(about = "Switch one collector on or off in the configuration, and on this host")]
    Collector(Collector),
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Collector {
    #[arg(value_name = "NAME", help = "Which collector")]
    pub name: String,

    #[command(subcommand)]
    pub doing: Switch,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Switch {
    #[command(about = "Watch it from now on")]
    Enable(Switching),

    #[command(about = "Stop watching it")]
    Disable(Switching),
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Switching {
    #[arg(
        long,
        value_name = "PATH",
        default_value = DEFAULT_PATH,
        help = "The configuration file to edit"
    )]
    pub config: String,

    #[arg(
        short = 'd',
        long,
        help = "Print what would be done, and do none of it"
    )]
    pub dry_run: bool,
}

impl Collector {
    pub fn switching(&self) -> &Switching {
        match &self.doing {
            Switch::Enable(switching) | Switch::Disable(switching) => switching,
        }
    }

    pub fn options(&self) -> collector::Options {
        collector::Options {
            name: self.name.clone(),
            path: self.switching().config.clone(),
            dry_run: self.switching().dry_run,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(styles = style::HELP)]
pub struct Configure {
    #[arg(value_name = "PATH", default_value = DEFAULT_PATH)]
    pub path: String,

    #[arg(
        short,
        long,
        help = "Replace a file that is already there",
        long_help = "\
Replace a file that is already there.

Without it an existing file is not touched. With it, what was there is kept
beside the new one as PATH.previous."
    )]
    pub force: bool,

    #[arg(
        short = 'd',
        long,
        help = "Print what would be written, to stdout, and write nothing",
        long_help = "\
Print what would be written, to stdout, and write nothing.

The output names which collectors would be switched off here."
    )]
    pub dry_run: bool,
}

impl From<Configure> for Options {
    fn from(asked: Configure) -> Self {
        Options {
            path: asked.path,
            force: asked.force,
            dry_run: asked.dry_run,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
