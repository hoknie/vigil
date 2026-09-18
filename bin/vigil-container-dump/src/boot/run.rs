use std::process::ExitCode;

use super::clock::now;
use super::dump::{Written, dump};
use crate::cli::{Command, Options, USAGE, parse};

pub fn start(arguments: impl Iterator<Item = String>) -> ExitCode {
    match parse(arguments) {
        Ok(Command::Run(options)) => run(&options),
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("vigil-container-dump {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("vigil-container-dump: {error}");
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

pub fn run(options: &Options) -> ExitCode {
    let written = match dump(options, &now()) {
        Ok(written) => written,
        Err(error) => {
            eprintln!("vigil-container-dump: {error}");
            return ExitCode::FAILURE;
        }
    };

    for one in &written {
        eprintln!("vigil-container-dump: {}", said(one));
    }

    ExitCode::SUCCESS
}

fn said(written: &Written) -> String {
    let Written { at, dump, .. } = written;
    let engine = &dump.engine;

    if !dump.on_this_host() {
        return format!(
            "{engine} is not installed here, and {} says so",
            at.display()
        );
    }

    let unanswered = dump.unanswered();
    let answered = dump.asked.len() - unanswered.len();
    match unanswered.is_empty() {
        true => format!(
            "{engine}: {answered} command(s) answered, written to {}",
            at.display()
        ),
        false => format!(
            "{engine}: {answered} command(s) answered and {} did not ({}), written to {}",
            unanswered.len(),
            unanswered.join("; "),
            at.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use vigil_engines::Dump;

    use super::*;

    fn workspace(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "vigil-container-dump-{}-{named}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&at);
        at
    }

    fn written(at: &Path) -> Dump {
        let text = fs::read_to_string(at.join("docker.json")).expect("a document per engine");
        serde_json::from_str(&text).expect("the document parses")
    }

    #[test]
    fn a_host_with_no_engine_on_it_gets_a_document_saying_absent_and_not_an_empty_file() {
        let at = workspace("absent");
        let options = Options {
            directory: at.display().to_string(),
            engines: vec![vigil_engines::Engine::Docker],
            ..Options::default()
        };

        assert_eq!(run(&options), ExitCode::SUCCESS);

        let dump = written(&at);
        if dump.on_this_host() {
            let _ = fs::remove_dir_all(&at);
            return;
        }
        assert_eq!(dump.state, "absent");
        assert!(
            dump.why.is_some_and(|why| why.contains("/usr/bin/docker")),
            "the document says where it looked, because a reader of it cannot look again"
        );
        assert!(dump.asked.is_empty());
        let _ = fs::remove_dir_all(&at);
    }

    #[test]
    fn the_document_is_written_whole_and_readable_only_by_the_account_that_wrote_it() {
        use std::os::unix::fs::PermissionsExt;

        let at = workspace("mode");
        let options = Options {
            directory: at.display().to_string(),
            engines: vec![vigil_engines::Engine::Docker],
            ..Options::default()
        };

        run(&options);

        let mode = fs::metadata(at.join("docker.json"))
            .expect("the document")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            mode, 0o600,
            "this file holds the labels of every container on the host, and a label is \
             whatever the person who wrote the compose file put in it"
        );
        assert!(
            !at.join("docker.json.writing").exists(),
            "the file the reader opens is renamed into place, so it never holds half a dump"
        );
        let _ = fs::remove_dir_all(&at);
    }
}
