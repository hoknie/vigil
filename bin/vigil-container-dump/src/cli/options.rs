use vigil_engines::{DUMP_DIRECTORY, Engine};

pub const DEADLINE_SECONDS: u64 = 10;

pub const CEILING_BYTES: u64 = 8 * 1024 * 1024;

pub const USAGE: &str = "\
usage: vigil-container-dump [--directory PATH] [--engine NAME] [--deadline SECONDS]
                            [--ceiling BYTES]

  Runs the listing commands of this host's container engines and writes what each one
  printed to one file per engine, 0600. vigild reads those files on its own round. It
  is started by vigil-containers.timer; by hand it writes the same files:

      vigil-container-dump --directory /tmp/dump

  It decides nothing, hides nothing and reports nothing: the reading, the projection and
  the redaction of secrets all happen in the agent, from what is written here.

  --directory PATH    where to write (default: /var/lib/vigil/containers)
  --engine NAME       docker or podman; given twice, both. The default is both, and an
                      engine that is not installed is written down as absent rather than
                      left out
  --deadline SECONDS  how long one command may take before it is stopped and recorded as
                      unfinished (default: 10)
  --ceiling BYTES     how much of one command's output is kept (default: 8388608). What
                      is over it is dropped and the file says so
  -v, --version       print the version and leave

  It opens no socket, reads no configuration and writes one file per engine.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub directory: String,
    pub engines: Vec<Engine>,
    pub deadline_seconds: u64,
    pub ceiling: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            directory: DUMP_DIRECTORY.to_string(),
            engines: Engine::ALL.to_vec(),
            deadline_seconds: DEADLINE_SECONDS,
            ceiling: CEILING_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Run(Options),
    Help,
    Version,
}

pub fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut options = Options::default();
    let mut named: Vec<Engine> = Vec::new();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-v" | "-V" => return Ok(Command::Version),
            "--directory" => {
                options.directory = arguments
                    .next()
                    .ok_or_else(|| "--directory needs a path".to_string())?;
            }
            "--engine" => {
                let word = arguments
                    .next()
                    .ok_or_else(|| "--engine needs docker or podman".to_string())?;
                let engine = Engine::named(&word)
                    .ok_or_else(|| format!("--engine wants docker or podman, not {word:?}"))?;
                if !named.contains(&engine) {
                    named.push(engine);
                }
            }
            "--deadline" => options.deadline_seconds = number(&mut arguments, "--deadline")?,
            "--ceiling" => options.ceiling = number(&mut arguments, "--ceiling")?,
            other => return Err(format!("unknown argument {other:?}")),
        }
    }

    if !named.is_empty() {
        options.engines = named;
    }
    if options.deadline_seconds == 0 {
        return Err("--deadline 0 stops every command before it starts".to_string());
    }

    Ok(Command::Run(options))
}

fn number(arguments: &mut impl Iterator<Item = String>, flag: &str) -> Result<u64, String> {
    let value = arguments
        .next()
        .ok_or_else(|| format!("{flag} needs a number"))?;

    value
        .parse()
        .map_err(|_| format!("{flag} wants a number, not {value:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_all(line: &[&str]) -> Result<Command, String> {
        parse(line.iter().map(|word| (*word).to_string()))
    }

    #[test]
    fn with_no_arguments_it_asks_both_engines_and_writes_where_the_agent_reads() {
        assert_eq!(
            parse_all(&[]).expect("parses"),
            Command::Run(Options {
                directory: DUMP_DIRECTORY.to_string(),
                engines: vec![Engine::Docker, Engine::Podman],
                deadline_seconds: DEADLINE_SECONDS,
                ceiling: CEILING_BYTES,
            })
        );
    }

    #[test]
    fn one_engine_may_be_named_and_naming_it_twice_asks_it_once() {
        let Command::Run(options) =
            parse_all(&["--engine", "podman", "--engine", "podman"]).expect("parses")
        else {
            panic!("a run");
        };

        assert_eq!(options.engines, vec![Engine::Podman]);
    }

    #[test]
    fn an_engine_this_build_never_heard_of_is_refused_by_name_rather_than_ignored() {
        let error = parse_all(&["--engine", "containerd"]).expect_err("must not be accepted");

        assert!(error.contains("containerd"), "{error}");
        assert!(error.contains("podman"), "{error}");
    }

    #[test]
    fn a_deadline_of_zero_is_refused_because_a_command_needs_to_be_able_to_answer() {
        assert!(parse_all(&["--deadline", "0"]).is_err());
        assert!(parse_all(&["--deadline", "10s"]).is_err());
    }

    #[test]
    fn the_version_is_asked_for_the_way_every_other_program_is_asked_for_it() {
        for spelling in ["-v", "-V", "--version"] {
            assert_eq!(parse_all(&[spelling]).expect("parses"), Command::Version);
        }
    }

    #[test]
    fn an_unknown_argument_is_refused_by_name_rather_than_ignored() {
        let error = parse_all(&["--socket"]).expect_err("must not be ignored");

        assert!(error.contains("--socket"), "{error}");
    }
}
