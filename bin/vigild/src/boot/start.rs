use std::process::ExitCode;

use clap::Parser;

use clap::CommandFactory;

use crate::cli::{Cli, Command, SUPPRESS_LIVES_IN_THE_CONSOLE, Switch};
use crate::{collector, wizard};

pub fn start(arguments: impl IntoIterator<Item = String>) -> ExitCode {
    let cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return match error.use_stderr() {
                true => ExitCode::from(2),
                false => ExitCode::SUCCESS,
            };
        }
    };

    match (cli.command, cli.config) {
        (Some(Command::Configure(options)), _) => match wizard::configure(&options.into()) {
            Ok(said) => {
                eprintln!("vigild configure: {said}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("vigild configure: {error}");
                ExitCode::FAILURE
            }
        },
        (Some(Command::Collector(asked)), _) => {
            let options = asked.options();
            let (doing, outcome) = match asked.doing {
                Switch::Enable(_) => ("enable", collector::enable(&options)),
                Switch::Disable(_) => ("disable", collector::disable(&options)),
            };
            match outcome {
                Ok(said) => {
                    eprintln!("vigild collector {} {doing}:\n  {said}", options.name);
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("vigild collector {} {doing}: {error}", options.name);
                    ExitCode::FAILURE
                }
            }
        }
        (Some(Command::Suppress { .. }), _) => {
            eprintln!("{SUPPRESS_LIVES_IN_THE_CONSOLE}");
            ExitCode::from(2)
        }
        (None, Some(path)) => match super::run(&path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("vigild: {error}");
                ExitCode::FAILURE
            }
        },
        (None, None) => {
            let _ = Cli::command().print_help();
            ExitCode::from(2)
        }
    }
}
