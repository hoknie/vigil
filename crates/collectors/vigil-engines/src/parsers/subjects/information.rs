use serde_json::{Value, json};

use crate::helpers::{flag, said, text, words};
use crate::types::{Engine, Subject};

const VERSION: &[&str] = &["ServerVersion", "version.Version", "Version"];

const STORAGE: &[&str] = &["Driver", "store.graphDriverName"];

const ROOT: &[&str] = &["DockerRootDir", "store.graphRoot"];

const CGROUP_DRIVER: &[&str] = &["CgroupDriver", "host.cgroupManager"];

const CGROUP_VERSION: &[&str] = &["CgroupVersion", "host.cgroupVersion"];

const LOGGING: &[&str] = &["LoggingDriver", "host.logDriver"];

const ROOTLESS: &[&str] = &["host.security.rootless", "Rootless"];

pub fn information(engine: Engine, present: bool, said_by_the_engine: Option<&Value>) -> Value {
    let Some(item) = said_by_the_engine else {
        return json!({
            "subject": Subject::Engine.as_str(),
            "engine": engine.name(),
            "present": present,
            "read": false,
            "version": Value::Null,
            "storage_driver": Value::Null,
            "root_directory": Value::Null,
            "cgroup_driver": Value::Null,
            "cgroup_version": Value::Null,
            "logging_driver": Value::Null,
            "rootless": Value::Null,
            "security_options": Vec::<String>::new(),
        });
    };

    json!({
        "subject": Subject::Engine.as_str(),
        "engine": engine.name(),
        "present": present,
        "read": true,
        "version": said(text(item, VERSION)),
        "storage_driver": said(text(item, STORAGE)),
        "root_directory": said(text(item, ROOT)),
        "cgroup_driver": said(text(item, CGROUP_DRIVER)),
        "cgroup_version": said(text(item, CGROUP_VERSION)),
        "logging_driver": said(text(item, LOGGING)),
        "rootless": match flag(item, ROOTLESS) {
            Some(rootless) => Value::Bool(rootless),
            None => Value::Null,
        },
        "security_options": words(item, &["SecurityOptions"]),
    })
}
