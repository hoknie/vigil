use vigil_firewall::MACOS_DUMP_DIRECTORY;

pub const DEADLINE_SECONDS: u64 = 10;

pub const CEILING_BYTES: u64 = 4 * 1024 * 1024;

pub const USAGE: &str = "\
usage: vigil-firewall-dump [--directory PATH] [--deadline SECONDS] [--ceiling BYTES]

  Asks pf and the Application Firewall of this Mac what they hold and writes what each
  answered to one file, firewall.json, 0600. vigild reads that file on its own round. It
  is started by the launchd job vigil.firewall, as root, because pfctl answers root alone;
  by hand it writes the same file:

      sudo vigil-firewall-dump --directory /tmp/firewall

  What it runs, and nothing else: /sbin/pfctl -s info, -s rules, -s nat, -v -s Anchors,
  and -a ANCHOR -s rules and -s nat for each anchor pfctl listed; and
  /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate --getblockall
  --getstealthmode --getallowsigned --listapps. Each with an empty environment, by its
  absolute path, never through a shell. It changes nothing: no flag it passes switches,
  loads, flushes or kills anything.

  --directory PATH    where to write (default: /usr/local/var/lib/vigil/firewall)
  --deadline SECONDS  how long one command may take before it is stopped and recorded as
                      unfinished (default: 10)
  --ceiling BYTES     how much of one command's output is kept (default: 4194304); what
                      is over it is dropped and the file says so
  -v, --version       print the version and leave

  It decides nothing and reports nothing: the reading is taken by the agent, from what
  is written here.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub directory: String,
    pub deadline_seconds: u64,
    pub ceiling: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            directory: MACOS_DUMP_DIRECTORY.to_string(),
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

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-v" | "-V" => return Ok(Command::Version),
            "--directory" => {
                options.directory = arguments
                    .next()
                    .ok_or_else(|| "--directory needs a path".to_string())?;
            }
            "--deadline" => options.deadline_seconds = number(&mut arguments, "--deadline")?,
            "--ceiling" => options.ceiling = number(&mut arguments, "--ceiling")?,
            other => return Err(format!("unknown argument {other:?}")),
        }
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
    fn with_no_arguments_it_writes_where_the_agent_on_a_mac_reads() {
        assert_eq!(
            parse_all(&[]).expect("parses"),
            Command::Run(Options {
                directory: "/usr/local/var/lib/vigil/firewall".to_string(),
                deadline_seconds: DEADLINE_SECONDS,
                ceiling: CEILING_BYTES,
            })
        );
    }

    #[test]
    fn a_deadline_of_zero_is_refused_because_a_command_needs_to_be_able_to_answer() {
        assert!(parse_all(&["--deadline", "0"]).is_err());
        assert!(parse_all(&["--deadline", "10s"]).is_err());
    }

    #[test]
    fn an_unknown_argument_is_refused_by_name_rather_than_ignored() {
        let error = parse_all(&["--flush"]).expect_err("must not be ignored");

        assert!(error.contains("--flush"), "{error}");
    }
}
