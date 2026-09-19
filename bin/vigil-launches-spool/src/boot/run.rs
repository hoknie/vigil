use std::io::BufReader;
use std::path::Path;
use std::process::ExitCode;

use vigil_launches::{
    ABSENT, ESLOGGER, ESLOGGER_SPOOL, ESLOGGER_STATUS, REFUSED, RUNNING, STOPPED, SpoolWriter,
    SpoolerStatus,
};

use super::clock::now;
use super::pump::{Pumped, pump};
use super::watching::{arguments, started};
use crate::cli::{Command, Options, USAGE, parse};
use crate::configuration::records_arguments;

const READ_AT_ONCE: usize = 256 * 1024;

pub fn start(arguments: impl Iterator<Item = String>) -> ExitCode {
    match parse(arguments) {
        Ok(Command::Run(options)) => run(&options, ESLOGGER),
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("vigil-launches-spool {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("vigil-launches-spool: {error}");
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

pub fn run(options: &Options, program: &str) -> ExitCode {
    let directory = Path::new(&options.directory);
    let status_path = directory.join(ESLOGGER_STATUS);

    let keep_arguments = match records_arguments(Path::new(&options.configuration)) {
        Ok(recorded) => recorded,
        Err(why) => {
            eprintln!("vigil-launches-spool: {why}. Launches are spooled without their arguments");
            false
        }
    };

    let mut spool = match SpoolWriter::open(directory.join(ESLOGGER_SPOOL), options.ceiling) {
        Ok(spool) => spool,
        Err(error) => {
            eprintln!(
                "vigil-launches-spool: cannot open the spool in {}: {error}",
                directory.display()
            );
            return ExitCode::FAILURE;
        }
    };

    let said = |state: &str, why: Option<String>| {
        let status = SpoolerStatus {
            state: state.to_string(),
            since: now(),
            program: program.to_string(),
            arguments_recorded: keep_arguments,
            why,
        };
        if let Err(error) = status.write(&status_path) {
            eprintln!(
                "vigil-launches-spool: cannot write {}: {error}",
                status_path.display()
            );
        }
    };

    if !Path::new(program).exists() {
        said(
            ABSENT,
            Some(format!(
                "{program} is not on this Mac; Endpoint Security is read through it from macOS 13"
            )),
        );
        eprintln!("vigil-launches-spool: {program} is not on this Mac");
        return ExitCode::FAILURE;
    }

    let mut watching = match started(program) {
        Ok(watching) => watching,
        Err(why) => {
            eprintln!("vigil-launches-spool: {why}");
            said(REFUSED, Some(why));
            return ExitCode::FAILURE;
        }
    };
    said(RUNNING, None);
    eprintln!(
        "vigil-launches-spool: {program} {} is running; arguments are {}",
        arguments(),
        match keep_arguments {
            true => "recorded, with secrets hidden first",
            false => "not recorded",
        }
    );

    let mut unwritable = None;
    let pumped = match watching.child.stdout.take() {
        Some(printed) => pump(
            &mut BufReader::with_capacity(READ_AT_ONCE, printed),
            &mut spool,
            keep_arguments,
            &mut unwritable,
        ),
        None => Pumped::default(),
    };

    let ended = watching.child.wait().ok().and_then(|status| status.code());
    let complaint = watching.complaint.join().unwrap_or_default();
    let why = match (complaint.is_empty(), ended) {
        (false, _) => complaint,
        (true, Some(code)) => format!("{program} exited with status {code} and said nothing"),
        (true, None) => format!("{program} was stopped by a signal and said nothing"),
    };

    let state = match pumped.events() == 0 && ended != Some(0) {
        true => REFUSED,
        false => STOPPED,
    };
    said(state, Some(why.clone()));

    let report = spool.report();
    eprintln!(
        "vigil-launches-spool: eslogger {state}: {why}. {} launch(es) written, {} with no \
         person behind them, {} other event(s), {} line(s) unreadable, {} compaction(s), {} \
         byte(s) dropped at the ceiling{}",
        pumped.written,
        pumped.nobody,
        pumped.other_events,
        pumped.unreadable,
        report.compactions,
        report.dropped,
        match unwritable {
            Some(error) => format!(", {} launch(es) unwritten ({error})", pumped.unwritten),
            None => String::new(),
        }
    );

    ExitCode::FAILURE
}
