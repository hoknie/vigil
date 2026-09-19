use vigil_config::Installation;
use vigil_launches::{CEILING_BYTES, LAUNCHES_DIRECTORY};

pub const USAGE: &str = "\
usage: vigil-launches-spool [--directory PATH] [--configuration PATH] [--ceiling BYTES]

  Keeps /usr/bin/eslogger running and writes every program a person launched on this Mac,
  as eslogger reports it, to a spool vigild reads on its own round. It is started and
  kept running by the launchd job vigil.launches, as root; eslogger also needs Full Disk
  Access, given to this program in System Settings, Privacy & Security.

  What it writes: the moment, the process, the person who logged in, the program. The
  arguments only when record_arguments is on in the launches block of the configuration,
  and then with secrets hidden before they are written. The environment never. A launch no
  person logged in for (a daemon launchd started) is not written at all.

  --directory PATH      where to keep the spool and eslogger's status
                        (default: /usr/local/var/lib/vigil/launches)
  --configuration PATH  the vigil.yaml whose launches block says whether arguments are
                        recorded (default: /usr/local/etc/vigil/vigil.yaml); a file that
                        cannot be read means they are not
  --ceiling BYTES       the size the spool is kept under, dropping the oldest launches
                        first and saying so (default: 16777216)
  -v, --version         print the version and leave

  It opens no socket and starts one program: eslogger, by its absolute path.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub directory: String,
    pub configuration: String,
    pub ceiling: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            directory: LAUNCHES_DIRECTORY.to_string(),
            configuration: Installation::MACOS.configuration.to_string(),
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

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-v" | "-V" => return Ok(Command::Version),
            "--directory" => {
                options.directory = arguments
                    .next()
                    .ok_or_else(|| "--directory needs a path".to_string())?;
            }
            "--configuration" => {
                options.configuration = arguments
                    .next()
                    .ok_or_else(|| "--configuration needs a path".to_string())?;
            }
            "--ceiling" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--ceiling needs a number".to_string())?;
                options.ceiling = value
                    .parse()
                    .map_err(|_| format!("--ceiling wants a number, not {value:?}"))?;
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }

    if options.ceiling == 0 {
        return Err("--ceiling 0 keeps nothing at all".to_string());
    }

    Ok(Command::Run(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_all(line: &[&str]) -> Result<Command, String> {
        parse(line.iter().map(|word| (*word).to_string()))
    }

    #[test]
    fn with_no_arguments_it_spools_where_the_agent_on_a_mac_reads_and_reads_its_configuration() {
        assert_eq!(
            parse_all(&[]).expect("parses"),
            Command::Run(Options {
                directory: "/usr/local/var/lib/vigil/launches".to_string(),
                configuration: "/usr/local/etc/vigil/vigil.yaml".to_string(),
                ceiling: CEILING_BYTES,
            })
        );
    }

    #[test]
    fn an_unknown_argument_is_refused_by_name_rather_than_ignored() {
        let error = parse_all(&["--arguments"]).expect_err("must not be ignored");

        assert!(error.contains("--arguments"), "{error}");
    }

    #[test]
    fn a_ceiling_of_nothing_is_refused() {
        assert!(parse_all(&["--ceiling", "0"]).is_err());
        assert!(parse_all(&["--ceiling", "16M"]).is_err());
    }
}
