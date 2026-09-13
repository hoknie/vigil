use std::path::Path;

use vigil_collect::Health;

use super::edit::{self, Edit};
use super::host::{self, Standing};
use crate::wizard::{DEFAULT_PATH, Surveyed, take, write};
use crate::{Config, config};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub name: String,
    pub path: String,
    pub dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            name: String::new(),
            path: DEFAULT_PATH.to_string(),
            dry_run: false,
        }
    }
}

const RESTART: &str = "systemctl try-restart vigild.service";

pub fn enable(options: &Options) -> Result<String, String> {
    let name = known(&options.name)?;
    let mut said = Vec::new();

    if let Some(unit) = crate::modules::unit_of(name) {
        said.push(start(unit, options.dry_run)?);
    }

    let standing = surveyed(name)?;
    if let Health::Unavailable(why) = &standing.health {
        return Err(format!(
            "{name} cannot read anything on this host, so nothing was written to {}.\n  {why}",
            options.path
        ));
    }

    said.push(written(options, name)?);

    if let Health::Degraded(why) = &standing.health {
        said.push(format!("{name} will read less than all of it: {why}"));
    }
    said.push(format!(
        "the daemon reads its configuration at start: {RESTART}"
    ));

    Ok(said.join("\n  "))
}

pub fn disable(options: &Options) -> Result<String, String> {
    let name = known(&options.name)?;
    let mut said = Vec::new();

    let text = read(&options.path)?;
    let every: Vec<(&str, u32)> = crate::modules::watched()
        .iter()
        .map(|collector| (collector.name, collector.every_seconds))
        .collect();

    match edit::remove(&text, name, &every) {
        Edit::NotOurs(why) => return Err(why),
        Edit::AlreadySo => said.push(format!("{name} is not named in {}", options.path)),
        Edit::Changed(after) => said.push(put(options, &text, &after)?),
    }

    match crate::modules::unit_of(name) {
        None => said.push(format!(
            "nothing on this host was started for {name}, so nothing was stopped"
        )),
        Some(unit) => match options.dry_run {
            true => said.push(format!(
                "would run: {} disable --now {unit}",
                host::SYSTEMCTL
            )),
            false => match host::standing(unit) {
                Standing::NoSystemd => said.push(format!(
                    "no systemd here, so {unit} was not stopped; whatever writes that reading \
                     on this host is yours to stop"
                )),
                _ => said.push(host::disable(unit)?),
            },
        },
    }

    said.push(
        "nothing else was touched: no unit this command did not enable, and no line of \
               the file it did not write"
            .to_string(),
    );
    said.push(format!(
        "the daemon reads its configuration at start: {RESTART}"
    ));

    Ok(said.join("\n  "))
}

fn start(unit: &str, dry_run: bool) -> Result<String, String> {
    if dry_run {
        return Ok(format!(
            "would run: {} enable --now {unit}",
            host::SYSTEMCTL
        ));
    }
    match host::standing(unit) {
        Standing::NoSystemd => Err(format!(
            "{unit} writes this reading and there is no systemd on this host ({} is not a \
             directory). Run `/usr/sbin/nft --json list ruleset > \
             /var/lib/vigil/firewall/ruleset.json` on a period of your own, and nothing here \
             was changed",
            host::SYSTEMD
        )),
        Standing::Masked => Err(format!(
            "{unit} is masked. Somebody switched this reading off deliberately and that \
             decision is older than this command: `systemctl unmask {unit}` first. Nothing \
             here was changed"
        )),
        Standing::Known(_) => host::enable(unit),
    }
}

fn written(options: &Options, name: &str) -> Result<String, String> {
    let text = read(&options.path)?;
    let every_seconds = crate::modules::every_seconds_of(name).unwrap_or(60);

    match edit::add(&text, name, every_seconds) {
        Edit::NotOurs(why) => Err(why),
        Edit::AlreadySo => Ok(match edit::watching(&text, name) {
            None => format!(
                "{} names no collectors at all, which means every one of them: {name} is \
                 already watched",
                options.path
            ),
            _ => format!("{name} is already named in {}", options.path),
        }),
        Edit::Changed(after) => put(options, &text, &after),
    }
}

fn put(options: &Options, before: &str, after: &str) -> Result<String, String> {
    if options.dry_run {
        println!("{after}");
        return Ok(format!(
            "nothing was written to {} (--dry-run)",
            options.path
        ));
    }

    let path = Path::new(&options.path);
    let written = write(path, after, true)?;
    config::load(&options.path).map_err(|error| {
        let _ = std::fs::write(path, before);
        format!("what this command wrote would not load, so the file was put back: {error}")
    })?;

    Ok(match &written.previous {
        Some(previous) => format!(
            "wrote {} (0600), and what was there is kept as {}",
            written.path.display(),
            previous.display()
        ),
        None => format!("wrote {} (0600)", written.path.display()),
    })
}

fn surveyed(name: &str) -> Result<Surveyed, String> {
    take(&Config::default())?
        .into_iter()
        .find(|collector| collector.name == name)
        .ok_or_else(|| format!("this build has no collector called {name:?}"))
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| {
        format!("{path}: {error}. `vigild configure` writes one this command can edit")
    })
}

fn known(name: &str) -> Result<&'static str, String> {
    crate::modules::watched()
        .iter()
        .find(|collector| collector.name == name)
        .map(|collector| collector.name)
        .ok_or_else(|| {
            format!(
                "no collector called {name:?}. This build has: {}",
                crate::modules::names().join(", ")
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_this_build_never_heard_of_is_refused_by_name_and_says_what_there_is() {
        let refused = known("firewal").expect_err("refused");

        assert!(refused.contains("firewal"), "{refused}");
        assert!(refused.contains("firewall"), "{refused}");
        assert!(refused.contains("ports"), "{refused}");
    }

    #[test]
    fn every_collector_this_build_ships_can_be_named_to_this_command() {
        for name in crate::modules::names() {
            assert_eq!(known(name), Ok(name));
        }
    }

    #[test]
    fn what_has_to_be_running_on_the_host_is_asked_of_the_collector_and_not_of_a_list_here() {
        assert_eq!(
            crate::modules::unit_of("firewall"),
            Some("vigil-firewall.timer")
        );
        assert_eq!(crate::modules::unit_of("ports"), None);

        let source = include_str!("run.rs");
        assert!(
            !source.contains("\"firewall\" =>") && !source.contains("== \"firewall\""),
            "a name compared against here is the second place to edit when the next \
             collector needs a unit, and the one that gets forgotten"
        );
    }
}
