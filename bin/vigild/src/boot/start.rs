use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, Command, NEEDS_A_CONFIGURATION};
use crate::wizard;

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
        (None, Some(path)) => match super::run(&path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("vigild: {error}");
                ExitCode::FAILURE
            }
        },
        (None, None) => {
            eprintln!("{NEEDS_A_CONFIGURATION}");
            ExitCode::from(2)
        }
    }
}
