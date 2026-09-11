use std::io::{self, Read};
use std::process::ExitCode;

use vigil_collect::SpoolWriter;

use crate::cli::{Command, Options, USAGE, parse};

const READ_CHUNK: usize = 64 * 1024;

pub fn start(arguments: impl Iterator<Item = String>) -> ExitCode {
    match parse(arguments) {
        Ok(Command::Run(options)) => run(&options, &mut io::stdin().lock()),
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("vigil-audit-plugin {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("vigil-audit-plugin: {error}");
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

pub fn run(options: &Options, events: &mut impl Read) -> ExitCode {
    let mut spool = match SpoolWriter::open(&options.spool_path, options.ceiling) {
        Ok(spool) => spool,
        Err(error) => {
            eprintln!(
                "vigil-audit-plugin: cannot open {}: {error}",
                options.spool_path
            );
            return ExitCode::FAILURE;
        }
    };

    let mut buffer = vec![0u8; READ_CHUNK];
    let mut unwritable: Option<String> = None;
    let mut lost = 0u64;

    loop {
        let read = match events.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                eprintln!("vigil-audit-plugin: reading the audit stream: {error}");
                break;
            }
        };

        if let Err(error) = spool.write(&buffer[..read]) {
            lost += read as u64;
            if unwritable.is_none() {
                eprintln!(
                    "vigil-audit-plugin: cannot write {}: {error}. Reading continues, so a full disk cannot back the kernel's audit queue up",
                    options.spool_path
                );
                unwritable = Some(error.to_string());
            }
        }
    }

    let report = spool.report();
    eprintln!(
        "vigil-audit-plugin: {} record(s) kept, {} not ours, {} compaction(s), {} byte(s) dropped at the ceiling{}",
        report.kept,
        report.skipped,
        report.compactions,
        report.dropped,
        match &unwritable {
            Some(error) => format!(", {lost} byte(s) unwritten ({error})"),
            None => String::new(),
        }
    );

    match unwritable {
        None => ExitCode::SUCCESS,
        Some(_) => ExitCode::FAILURE,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    const EVENT: &str = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 auid=1000 uid=1000 comm="nc" exe="/usr/bin/nc" key="vigil_exec""#,
        "\n",
        r#"type=EXECVE msg=audit(1757419203.412:3421): argc=2 a0="nc" a1="-l""#,
        "\n",
        "type=PROCTITLE msg=audit(1757419203.412:3421): proctitle=6E63002D6C\n",
    );

    fn workspace(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join("vigil-plugin-test");
        fs::create_dir_all(&directory).expect("temp dir");
        let path = directory.join(format!("{}-{name}", std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn a_recorded_log_on_stdin_becomes_a_spool_the_collector_can_read() {
        let path = workspace("recorded");
        let options = Options {
            spool_path: path.display().to_string(),
            ceiling: 1024 * 1024,
        };

        let code = run(&options, &mut EVENT.repeat(3).as_bytes());

        assert_eq!(code, ExitCode::SUCCESS);
        let written = fs::read_to_string(&path).expect("readable");
        assert_eq!(written.lines().count(), 6, "two records of each event");
        assert!(!written.contains("PROCTITLE"), "{written}");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_spool_it_cannot_open_is_the_one_thing_that_stops_it() {
        let options = Options {
            spool_path: std::env::temp_dir().display().to_string(),
            ceiling: 1024,
        };

        assert_eq!(run(&options, &mut EVENT.as_bytes()), ExitCode::FAILURE);
    }
}
