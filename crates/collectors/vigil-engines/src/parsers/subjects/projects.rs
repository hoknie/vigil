use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::helpers::{CONFIGURATION_FILES, PROJECT, SERVICE, WORKING_DIRECTORY, labels, of, said};
use crate::types::Subject;

#[derive(Default)]
struct Gathered {
    services: Vec<String>,
    containers: Vec<String>,
    working_directory: Option<String>,
    configuration_files: Vec<String>,
}

pub fn projects(rows: &[Value], containers: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut gathered: BTreeMap<String, Gathered> = BTreeMap::new();

    for row in rows {
        let written = labels(row, &["Labels", "labels"]);
        let Some(project) = of(&written, PROJECT) else {
            continue;
        };
        let held = gathered.entry(project).or_default();

        if let Some(service) = of(&written, SERVICE) {
            held.services.push(service);
        }
        if held.working_directory.is_none() {
            held.working_directory = of(&written, WORKING_DIRECTORY);
        }
        for file in of(&written, CONFIGURATION_FILES)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|file| !file.is_empty())
        {
            held.configuration_files.push(file.to_string());
        }
    }

    for (name, held) in &mut gathered {
        held.containers = containers
            .iter()
            .filter(|(_, item)| item["project"] == Value::String(name.clone()))
            .map(|(named, _)| named.clone())
            .collect();
        settled(&mut held.services);
        settled(&mut held.configuration_files);
        settled(&mut held.containers);
    }

    gathered
        .into_iter()
        .map(|(name, held)| {
            let row = json!({
                "subject": Subject::Project.as_str(),
                "name": name.clone(),
                "services": held.services,
                "containers": held.containers,
                "working_directory": said(held.working_directory),
                "configuration_files": held.configuration_files,
            });
            (name, row)
        })
        .collect()
}

fn settled(listed: &mut Vec<String>) {
    listed.sort_unstable();
    listed.dedup();
}
