use vigil_collect::{CEILING_BYTES, SPOOL_PATH};

pub const USAGE: &str = "\
usage: vigil-audit-plugin [--spool PATH] [--ceiling BYTES]

  Reads audit records on standard input and appends them to vigil's spool, where the
  daemon's launches collector picks them up. auditd starts it, registered in
  /etc/audit/plugins.d/vigil.conf. By hand, it replays a recorded log:

      vigil-audit-plugin --spool /tmp/spool < audit.log

  --spool PATH     where to write (default: /var/lib/vigil/audit-spool)
  --ceiling BYTES  how much the spool may hold before the oldest events in it are
                   dropped (default: 16777216). Reaching it is reported as a finding
  -v, --version    print the version and leave

  It makes no decision, opens no socket and reads no configuration. It writes one file.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub spool_path: String,
    pub ceiling: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            spool_path: SPOOL_PATH.to_string(),
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
            "--spool" => {
                options.spool_path = arguments
                    .next()
                    .ok_or_else(|| "--spool needs a path".to_string())?;
            }
            "--ceiling" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--ceiling needs a number of bytes".to_string())?;
                options.ceiling = value
                    .parse()
                    .map_err(|_| format!("--ceiling wants a number of bytes, not {value:?}"))?;
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
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
    fn with_no_arguments_it_writes_the_spool_the_collector_reads() {
        assert_eq!(
            parse_all(&[]).expect("parses"),
            Command::Run(Options {
                spool_path: SPOOL_PATH.to_string(),
                ceiling: CEILING_BYTES,
            })
        );
    }

    #[test]
    fn a_spool_and_a_ceiling_can_both_be_named() {
        assert_eq!(
            parse_all(&["--spool", "/tmp/spool", "--ceiling", "4096"]).expect("parses"),
            Command::Run(Options {
                spool_path: "/tmp/spool".to_string(),
                ceiling: 4096,
            })
        );
    }

    #[test]
    fn the_version_is_asked_for_the_way_every_other_program_is_asked_for_it() {
        for spelling in ["-v", "-V", "--version"] {
            assert_eq!(
                parse_all(&[spelling]).expect("parses"),
                Command::Version,
                "vigil-audit-plugin {spelling} did not ask for the version"
            );
        }
    }

    #[test]
    fn an_unknown_argument_is_refused_by_name_rather_than_ignored() {
        let error = parse_all(&["--socket"]).expect_err("must not be ignored");

        assert!(error.contains("--socket"), "{error}");
    }

    #[test]
    fn a_ceiling_that_is_not_a_number_says_so_instead_of_becoming_a_default() {
        let error = parse_all(&["--ceiling", "16M"]).expect_err("must not be accepted");

        assert!(error.contains("16M"), "{error}");
    }
}
