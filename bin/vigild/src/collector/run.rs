use vigil_collect::Health;

use super::apart::{collectors_at, switch};
use super::edit::{self, Edit};
use super::host::{self, Standing};
use super::manager::Manager;
use crate::wizard::{DEFAULT_PATH, Surveyed, take};
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

pub fn enable(options: &Options) -> Result<String, String> {
    let name = known(&options.name)?;
    let mut said = Vec::new();
    let text = read(&options.path)?;
    let apart = collectors_at(options, &text)?;

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

    said.push(match &apart {
        Some(at) => switch(options, name, at, true)?,
        None => written(options, &text, name)?,
    });

    if let Health::Degraded(why) = &standing.health {
        said.push(format!("{name} will read less than all of it: {why}"));
    }
    said.push(config::restart_note());

    Ok(said.join("\n  "))
}

pub fn disable(options: &Options) -> Result<String, String> {
    let name = known(&options.name)?;
    let mut said = Vec::new();

    let text = read(&options.path)?;
    match collectors_at(options, &text)? {
        Some(at) => said.push(switch(options, name, &at, false)?),
        None => said.push(unnamed(options, &text, name)?),
    }

    match crate::modules::unit_of(name) {
        None => said.push(format!(
            "nothing on this host was started for {name}, so nothing was stopped"
        )),
        Some(unit) => match options.dry_run {
            true => said.push(format!("would run: {}", host::said_disabling(unit))),
            false => match host::standing(unit) {
                Standing::NoManager(why) => said.push(format!(
                    "{unit} was not stopped: {why}. Whatever writes that reading on this host \
                     is yours to stop"
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
    said.push(config::restart_note());

    Ok(said.join("\n  "))
}

fn start(unit: &str, dry_run: bool) -> Result<String, String> {
    if dry_run {
        return Ok(format!("would run: {}", host::said_enabling(unit)));
    }
    match host::standing(unit) {
        Standing::NoManager(why) => Err(format!(
            "{unit} writes this reading and {why}. Nothing here was changed"
        )),
        Standing::Masked => Err(format!(
            "{unit} is masked. Somebody switched this reading off deliberately and that \
             decision is older than this command: `{}` first. Nothing here was changed",
            Manager::here()
                .masking_undone(unit)
                .unwrap_or_else(|| format!("undo what switched {unit} off"))
        )),
        Standing::Known(_) => host::enable(unit),
    }
}

fn unnamed(options: &Options, text: &str, name: &str) -> Result<String, String> {
    let every: Vec<(&str, u32)> = crate::modules::watched()
        .iter()
        .map(|collector| (collector.name, collector.every_seconds))
        .collect();

    match edit::remove(text, listed_as(text, name), &every) {
        Edit::NotOurs(why) => Err(why),
        Edit::AlreadySo => Ok(format!("{name} is not named in {}", options.path)),
        Edit::Changed(after) => put(options, text, &after),
    }
}

fn written(options: &Options, text: &str, name: &str) -> Result<String, String> {
    let every_seconds = crate::modules::every_seconds_of(name).unwrap_or(60);

    match edit::add(text, listed_as(text, name), every_seconds) {
        Edit::NotOurs(why) => Err(why),
        Edit::AlreadySo => Ok(match edit::watching(text, name) {
            None => format!(
                "{} names no collectors at all, which means every one of them: {name} is \
                 already watched",
                options.path
            ),
            _ => format!("{name} is already named in {}", options.path),
        }),
        Edit::Changed(after) => put(options, text, &after),
    }
}

fn listed_as<'a>(text: &str, name: &'a str) -> &'a str {
    match config::former_name(name) {
        Some(was)
            if edit::watching(text, name) != Some(true)
                && edit::watching(text, was) == Some(true) =>
        {
            was
        }
        _ => name,
    }
}

fn put(options: &Options, before: &str, after: &str) -> Result<String, String> {
    config::put(&options.path, before, after, options.dry_run)
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
        assert!(refused.contains("network"), "{refused}");
    }

    #[test]
    fn a_file_of_the_former_layout_that_lists_ports_is_edited_under_the_name_it_wrote() {
        let former = "collectors:\n  - ports\n  - users\n";

        assert_eq!(
            listed_as(former, "network"),
            "ports",
            "switching network off on an upgraded host takes out the line that switched it on"
        );
        assert_eq!(
            listed_as("collectors:\n  - network\n", "network"),
            "network"
        );
        assert_eq!(listed_as(former, "users"), "users");
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
        assert_eq!(crate::modules::unit_of("users"), None);

        let source = include_str!("run.rs");
        assert!(
            !source.contains("\"firewall\" =>") && !source.contains("== \"firewall\""),
            "a name compared against here is the second place to edit when the next \
             collector needs a unit, and the one that gets forgotten"
        );
    }
}
