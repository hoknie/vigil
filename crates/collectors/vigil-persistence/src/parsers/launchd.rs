use super::plist::PlistValue;

const INSERTED: &str = "DYLD_INSERT_LIBRARIES";

const MOST_ARGUMENTS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LaunchdFacts {
    pub label: Option<String>,
    pub program: Option<String>,
    pub arguments: Vec<String>,
    pub user_name: Option<String>,
    pub run_at_load: bool,
    pub keep_alive: bool,
    pub start_interval: Option<i128>,
    pub calendar: bool,
    pub watch_paths: Vec<String>,
    pub sockets: bool,
    pub disabled: bool,
    pub inserted_libraries: Vec<String>,
}

impl LaunchdFacts {
    pub fn schedule(&self) -> String {
        let mut said: Vec<String> = Vec::new();
        if self.run_at_load {
            said.push("at load".to_string());
        }
        if self.keep_alive {
            said.push("kept alive".to_string());
        }
        if let Some(seconds) = self.start_interval {
            said.push(format!("every {seconds} s"));
        }
        if self.calendar {
            said.push("on a calendar".to_string());
        }
        for path in &self.watch_paths {
            said.push(format!("when {path} changes"));
        }
        if self.sockets {
            said.push("on a connection".to_string());
        }
        match said.is_empty() {
            true => "on request".to_string(),
            false => said.join(", "),
        }
    }
}

pub fn launchd_facts(job: &PlistValue) -> LaunchdFacts {
    let text = |key: &str| {
        job.get(key)
            .and_then(PlistValue::as_str)
            .map(str::to_string)
    };
    let flag = |key: &str| job.get(key).and_then(PlistValue::as_bool).unwrap_or(false);
    let strings = |value: Option<&PlistValue>| -> Vec<String> {
        value
            .and_then(PlistValue::as_array)
            .unwrap_or_default()
            .iter()
            .take(MOST_ARGUMENTS)
            .filter_map(PlistValue::as_str)
            .map(str::to_string)
            .collect()
    };

    let listed = strings(job.get("ProgramArguments"));
    let program = text("Program")
        .or_else(|| text("BundleProgram"))
        .or_else(|| listed.first().cloned());
    let arguments = match (listed.is_empty(), &program) {
        (false, _) => listed,
        (true, Some(program)) => vec![program.clone()],
        (true, None) => Vec::new(),
    };

    LaunchdFacts {
        label: text("Label"),
        program,
        arguments,
        user_name: text("UserName"),
        run_at_load: flag("RunAtLoad"),
        keep_alive: match job.get("KeepAlive") {
            Some(PlistValue::Boolean(kept)) => *kept,
            Some(conditions) => conditions.is_dictionary(),
            None => false,
        },
        start_interval: job.get("StartInterval").and_then(PlistValue::as_integer),
        calendar: job.get("StartCalendarInterval").is_some(),
        watch_paths: strings(job.get("WatchPaths"))
            .into_iter()
            .chain(strings(job.get("QueueDirectories")))
            .collect(),
        sockets: job.get("Sockets").is_some(),
        disabled: flag("Disabled"),
        inserted_libraries: job
            .get("EnvironmentVariables")
            .and_then(|environment| environment.get(INSERTED))
            .and_then(PlistValue::as_str)
            .map(|inserted| {
                inserted
                    .split(':')
                    .filter(|library| !library.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    }
}
