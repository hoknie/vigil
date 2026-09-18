use vigil_launches::Watching;

use super::prose::{WIDTH, comment, indented_comment, wrap};
use super::{Surveyed, periods};
use crate::Config;

pub fn configuration(taken_at: &str, survey: &[Surveyed], defaults: &Config) -> String {
    let mut out = String::new();

    out.push_str(&preamble(taken_at));
    out.push('\n');
    out.push_str(&format!("state_dir: {}\n", defaults.state_dir));
    out.push_str(&format!("socket_path: {}\n", defaults.socket_path));
    out.push_str(&format!("retention_days: {}\n", defaults.retention_days));
    out.push('\n');
    out.push_str(&collectors(taken_at, survey));
    out.push('\n');
    out.push_str(&periods::schedule(survey, defaults));
    out.push('\n');
    out.push_str(REPORTERS);
    out.push('\n');
    out.push_str(SUPPRESSIONS);
    out.push('\n');
    out.push_str(&killing(defaults));
    out.push('\n');
    out.push_str(&accounts(defaults));
    out.push('\n');
    out.push_str(&units(defaults));
    out.push('\n');
    out.push_str(&arguments());

    out
}

fn preamble(taken_at: &str) -> String {
    let mut out = String::new();
    for line in [
        format!("vigil: the configuration of this host, written by `vigild configure` at {taken_at}."),
        String::new(),
        "This is a reading of THIS host, not a template. Every collector below was asked whether it can run here, and the answer put its name in the list or in a comment. Ask again without writing anything: `vigild configure --dry-run`.".into(),
        String::new(),
        "Every value here is also its default, so a removed line changes nothing. The one exception is the list of collectors: see the note above it.".into(),
        String::new(),
        "Under the shipped systemd unit this agent has a /tmp and a /var/tmp of its own (PrivateTmp=yes). A path under either reads as absent even when the host has a file there, so a binary run from /tmp can be reported as no longer on disk. Drop PrivateTmp with `systemctl edit vigild` to watch those two directories.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out
}

fn collectors(taken_at: &str, survey: &[Surveyed]) -> String {
    let mut out = String::new();
    for line in [
        "What is watched. ONLY WHAT IS NAMED HERE RUNS.".to_string(),
        String::new(),
        "Take a name out and that collector stops. It is named on start-up and in the console, and the reading it kept is dropped: put the name back later and the next reading is a fresh baseline.".into(),
        String::new(),
        "Remove the whole key and every collector this build has runs. An empty list watches nothing. That is supported, and it is said on start-up.".into(),
        String::new(),
        format!("Asked on this host at {taken_at}:"),
    ] {
        out.push_str(&comment(&line));
    }

    out.push_str("collectors:\n");
    for collector in survey.iter().filter(|collector| collector.runs_here()) {
        out.push_str(&indented_comment(&format!(
            "{} — {}",
            collector.state(),
            collector.subject
        )));
        if let Some(reason) = collector.reason() {
            for line in wrap(reason, WIDTH - 8) {
                out.push_str(&format!("  #     {line}\n"));
            }
        }
        out.push_str(&format!("  - {}\n", collector.name));
    }

    let absent: Vec<&Surveyed> = survey
        .iter()
        .filter(|collector| !collector.runs_here())
        .collect();
    if !absent.is_empty() {
        out.push('\n');
        for collector in absent {
            out.push_str(&indented_comment(&format!(
                "NOT ENABLED: {} cannot run on this host, so nothing watches {}:",
                collector.name, collector.subject
            )));
            for line in wrap(
                collector.reason().unwrap_or("no reason was given"),
                WIDTH - 8,
            ) {
                out.push_str(&format!("  #     {line}\n"));
            }
            out.push_str(&indented_comment(
                "Put that right and take the # off the line below.",
            ));
            out.push_str(&format!("  #  - {}\n", collector.name));
        }
    }
    out
}

const REPORTERS: &str = "\
# Where findings go. They are kept apart from this file, in the files under reporters_path, so
# that writing this file again never loses a receiver somebody set up. Every file there that
# ends in .yaml or .yml is read, in the order of their names, and each holds a list of its own;
# a list written in this file is read as well. None at all is supported: the console reads the
# local history, and on a host with no network out nothing else is needed. A file there may
# name a file with a token in it, so the directory is root's alone.
#
#   /etc/vigil/reporters/journal.yaml:
#     reporters:
#       - kind: ndjson
#         path: /var/log/vigil/findings.ndjson
#       - kind: syslog
#         facility: local0
reporters_path: /etc/vigil/reporters
";

const SUPPRESSIONS: &str = "\
# What this host is expected to do, and therefore what not to report. There is no learning
# window and no grace period: the first reading of a collector is a baseline and produces
# nothing, and everything after it is reported unless an entry says otherwise. Each entry
# needs a reason, in your words.
#
# The entries are kept apart from this file, in the files under suppressions_path, so that
# writing this file again never loses a year of them. Every file there that ends in .yaml or
# .yml is read, in the order of their names; keep one per purpose if that helps. The console
# and `vigil suppress add` write to console.yaml there, `vigil suppress list` reads every
# file back, `vigil suppress remove` takes an entry out of whichever file holds it.
#
#   /etc/vigil/suppressions/deploy.yaml:
#     suppressions:
#       - finding_key: \"port.listen|tcp|0.0.0.0:8080\"
#         reason: the staging api, expected on this host
suppressions_path: /etc/vigil/suppressions
";

fn killing(defaults: &Config) -> String {
    let mut out = String::new();
    for line in [
        "Whether a person at the console of this host may ask the agent to close a listening socket.".to_string(),
        String::new(),
        "Off by default, and one of the three keys in this file that let the agent change anything on a host it did not set up. On, the console can ask for one of three things against the sockets a person marked there: SIGTERM to the process holding one, SIGKILL to it, or closing the socket itself and leaving the process running. The console asks the person to confirm; the agent asks nothing and does what it was told.".into(),
        String::new(),
        "Everything it does, and everything it refuses to do, is a finding of its own, so what happened is in the journal and at whatever receiver this file names. The agent will not signal pid 1 and will not signal itself.".into(),
        String::new(),
        "Nothing reaches this from the network. The console socket is 0600 and local; whoever can read it can already read every process on this host. Turning this on gives that account one more thing: it can stop them.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out.push_str("killing:\n");
    out.push_str(&format!(
        "  from_the_console: {}\n",
        defaults.killing.from_the_console
    ));
    out
}

fn accounts(defaults: &Config) -> String {
    let mut out = String::new();
    for line in [
        "Whether a person at the console of this host may ask the agent to change its accounts.".to_string(),
        String::new(),
        "Off by default, and separate from killing: a host where a program may be stopped has not thereby agreed that sudo may be granted. On, the console can ask the agent to change an account (shell, home, comment, lock, groups) or delete it, keeping its home directory; create, change or delete a group; change or remove a sudo grant; add, change or remove a key in authorized_keys; and end a session. The agent does it with the system's own tools, useradd's family, gpasswd, visudo and loginctl, by absolute path.".into(),
        String::new(),
        "Everything it does, and everything it refuses to do, is a finding of its own. It will not delete or lock uid 0, delete the account it runs as, or delete the group with gid 0. Sudo grants are written only in /etc/sudoers.d, and a file that visudo does not accept never replaces the one in place; a grant in /etc/sudoers itself is left for a person to edit.".into(),
        String::new(),
        "Nothing reaches this from the network. The console socket is 0600 and local; turning this on gives whoever can read it the accounts of this host.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out.push_str("accounts:\n");
    out.push_str(&format!(
        "  from_the_console: {}\n",
        defaults.accounts.from_the_console
    ));
    out
}

fn units(defaults: &Config) -> String {
    let mut out = String::new();
    for line in [
        "Whether a person at the console of this host may ask the agent to start and stop what this host starts by itself.".to_string(),
        String::new(),
        "Off by default, and a key of its own: a host where a program may be signalled has not thereby agreed that a service may be disabled until somebody notices. On, the console can ask for one of six things against the units and timers a person marked on the startup screen — stop, start, disable, enable, mask, unmask — each of which the same band undoes, and for one of two against a cron job: comment its line out, or take the # off again. The agent hands systemctl the word and the unit name it read off this host, and nothing else: no flags, no shell, no restart, no reboot, no daemon-reload.".into(),
        String::new(),
        "A crontab is rewritten in place, by a new file renamed over it, keeping its owner and its mode, following no symbolic link, and only when the line the console marked is still in it, byte for byte. A job that is a whole file of /etc/cron.daily and its neighbours has no line to comment out and is refused by name.".into(),
        String::new(),
        "Everything it does, and everything it refuses to do, is a finding of its own. The agent will not stop itself, nor a unit one of its own readings needs.".into(),
        String::new(),
        "Nothing reaches this from the network. The console socket is 0600 and local; turning this on gives whoever can read it the services of this host.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out.push_str("units:\n");
    out.push_str(&format!(
        "  from_the_console: {}\n",
        defaults.units.from_the_console
    ));
    out
}

fn arguments() -> String {
    let mut out = String::new();
    for line in [
        "Whether a finding about a program launch may carry the arguments the command was given.".to_string(),
        String::new(),
        "Off by default. Off means the arguments are never assembled, so they reach neither the local history nor a receiver. On, secrets are removed first, and that removal is not complete. The shipped example in /etc/vigil says what is at stake.".into(),
    ] {
        out.push_str(&comment(&line));
    }
    out.push_str("launches:\n");
    out.push_str(&format!(
        "  record_arguments: {}\n",
        Watching::default().record_arguments
    ));
    out
}
