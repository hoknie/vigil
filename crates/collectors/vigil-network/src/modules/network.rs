use serde::Deserialize;
use vigil_collect::Collector;
use vigil_module::{Module, Settings};
use vigil_rules::RuleSet;
use vigil_view::Section;

use crate::rules::listening_port_rules;
use crate::views::Listening;

const FAMILIES: &[&str] = &["port.listen", "connection.outbound"];

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Reading {
    pub connections: bool,
}

impl Default for Reading {
    fn default() -> Reading {
        Reading { connections: true }
    }
}

pub struct Network;

impl Module for Network {
    fn name(&self) -> &'static str {
        "network"
    }

    fn subject(&self) -> &'static str {
        "the sockets this host listens on, and the process holding each one"
    }

    fn every_seconds(&self) -> u32 {
        30
    }

    fn settings_key(&self) -> Option<&'static str> {
        Some("network")
    }

    fn check(&self, settings: &Settings) -> Result<(), String> {
        let said: serde_json::Value = settings.read().map_err(|refusal| refusal.to_string())?;
        if let Some(connections) = said.get("connections")
            && !connections.is_boolean()
        {
            return Err(format!(
                "connections: {connections} is neither true nor false; true reads the pairs \
                 of programs and peers, false reads the listening sockets alone"
            ));
        }

        settings
            .read::<Reading>()
            .map(|_| ())
            .map_err(|refusal| refusal.to_string())
    }

    fn collector(&self, settings: &Settings) -> Result<Box<dyn Collector>, String> {
        let reading: Reading = settings.read().map_err(|refusal| refusal.to_string())?;

        Ok(Box::new(
            crate::NetworkCollector::new(settings.now()).reading_connections(reading.connections),
        ))
    }

    fn rules(&self, _settings: &Settings) -> RuleSet {
        listening_port_rules()
    }

    fn section(&self) -> Option<Box<dyn Section>> {
        Some(Box::new(Listening))
    }

    fn families(&self) -> &[&'static str] {
        FAMILIES
    }

    fn row_of(&self, finding_key: &str) -> Option<String> {
        let (family, rest) = finding_key.split_once('|')?;
        match family {
            "port.listen" => Some(rest.to_string()),
            "connection.outbound" => Some(format!("outbound|{rest}")),
            _ => None,
        }
    }
}
