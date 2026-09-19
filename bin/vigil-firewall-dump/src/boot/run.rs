use std::process::ExitCode;

use super::clock::now;
use super::dump::dump;
use crate::cli::{Command, Options, USAGE, parse};

pub fn start(arguments: impl Iterator<Item = String>) -> ExitCode {
    match parse(arguments) {
        Ok(Command::Run(options)) => run(&options),
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("vigil-firewall-dump {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("vigil-firewall-dump: {error}");
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

pub fn run(options: &Options) -> ExitCode {
    if !cfg!(target_os = "macos") {
        eprintln!(
            "vigil-firewall-dump: pf and the Application Firewall are read on macOS; on Linux \
             the vigil-firewall.timer unit runs nft instead"
        );
        return ExitCode::from(2);
    }

    let (at, written) = match dump(options, &now()) {
        Ok(written) => written,
        Err(error) => {
            eprintln!("vigil-firewall-dump: {error}");
            return ExitCode::FAILURE;
        }
    };

    let unanswered: Vec<String> = written
        .asked
        .iter()
        .filter(|(_, answer)| !answer.answered())
        .map(|(key, answer)| format!("{key}: {}", answer.shortly()))
        .collect();
    match unanswered.is_empty() {
        true => eprintln!(
            "vigil-firewall-dump: {} question(s) answered, written to {}",
            written.asked.len(),
            at.display()
        ),
        false => eprintln!(
            "vigil-firewall-dump: {} question(s) answered and {} not ({}), written to {}",
            written.asked.len() - unanswered.len(),
            unanswered.len(),
            unanswered.join("; "),
            at.display()
        ),
    }

    ExitCode::SUCCESS
}
