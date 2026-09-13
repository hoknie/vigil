use super::arguments::arguments;
use super::execution::Execution;
use super::fields::{number, value};
use super::log::AUDIT_KEY;
use super::record::Record;
use super::text::{text, unquote};

pub(super) struct Event {
    id: String,
    pub(super) started_at: usize,
    pub(super) last_line: usize,
    syscall: Vec<(String, String)>,
    execve: Vec<(String, String)>,
    working_directory: Option<String>,
    first_path: Option<String>,
    pub(super) succeeded: bool,
}

impl Event {
    pub(super) fn new(id: String, started_at: usize) -> Self {
        Event {
            id,
            started_at,
            last_line: 0,
            syscall: Vec::new(),
            execve: Vec::new(),
            working_directory: None,
            first_path: None,
            succeeded: false,
        }
    }

    pub(super) fn absorb(&mut self, record: Record) {
        match record.kind.as_str() {
            "SYSCALL" => {
                self.succeeded = value(&record.fields, "success")
                    .map(|raw| raw == "yes")
                    .unwrap_or(false);
                self.syscall = record.fields;
            }
            "EXECVE" => self.execve = record.fields,
            "CWD" => {
                self.working_directory = value(&record.fields, "cwd").and_then(|raw| text(raw).0)
            }
            "PATH"
                if self.first_path.is_none()
                    && value(&record.fields, "nametype")
                        .is_none_or(|kind| unquote(kind) == "NORMAL") =>
            {
                self.first_path = value(&record.fields, "name").and_then(|raw| text(raw).0);
            }
            _ => {}
        }
    }

    pub(super) fn complete(&self) -> bool {
        !self.syscall.is_empty() && !self.execve.is_empty()
    }

    pub(super) fn half_written(&self) -> bool {
        self.syscall.is_empty() != self.execve.is_empty()
    }

    pub(super) fn ours(&self) -> bool {
        match value(&self.syscall, "key") {
            Some(raw) => text(raw).0.is_some_and(|key| key.contains(AUDIT_KEY)),
            None => false,
        }
    }

    pub(super) fn execution(&self, with_arguments: bool) -> Option<Execution> {
        let (executable, lossy) = match value(&self.syscall, "exe") {
            Some(raw) => text(raw),
            None => (
                self.first_path
                    .as_ref()
                    .map(|path| match path.starts_with('/') {
                        true => path.clone(),
                        false => format!(
                            "{}/{path}",
                            self.working_directory.clone().unwrap_or_default()
                        ),
                    }),
                false,
            ),
        };
        let executable = executable.filter(|path| path.starts_with('/'))?;

        let (arguments, arguments_lossy) = match with_arguments {
            true => arguments(&self.execve),
            false => (Vec::new(), false),
        };

        Some(Execution {
            id: self.id.clone(),
            auid: number(&self.syscall, "auid"),
            executable: Some(executable),
            executable_lossy: lossy,
            arguments,
            arguments_lossy,
        })
    }
}
