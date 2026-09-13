mod collectors;
pub mod fixture;
mod modules;
#[cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]
mod parsers;
mod rules;
mod types;
mod views;

pub use modules::Persistence;
pub use views::WhatStartsByItself;

#[cfg(target_os = "linux")]
pub use collectors::PersistenceCollector;
