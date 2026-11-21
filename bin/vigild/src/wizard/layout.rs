use vigil_config::{Homebrew, Installation, Layout, NO_HOME};

use super::launch_agent;
use crate::helpers::account;

pub fn here() -> Result<Layout, String> {
    Layout::for_account(Installation::here(), &account::this())
        .map(|layout| layout.under(Homebrew::of(&launch_agent::this_program()).as_ref()))
}

pub fn default_path() -> String {
    here()
        .map(|layout| layout.configuration)
        .unwrap_or_default()
}

pub fn named(path: &str) -> Result<(), String> {
    match path.is_empty() {
        true => Err(format!("{NO_HOME}: name the configuration file to write")),
        false => Ok(()),
    }
}

pub fn per_account() -> bool {
    here().is_ok_and(|layout| layout.per_account)
}

pub fn restart_words() -> String {
    here()
        .map(|layout| layout.restart)
        .unwrap_or_else(|_| Installation::here().service.restart.to_string())
}

pub fn job_left_by_the_archive() -> Option<std::path::PathBuf> {
    let home = account::this().home?;
    Layout::account_launch_agent(&home).map(std::path::PathBuf::from)
}
