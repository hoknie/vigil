use vigil_collect::{
    CollectError, Program, arguments_buffer, arguments_of, executable_of, parse_process_arguments,
    processes_running, program_of, redact,
};

use crate::parsers::ProcessRow;

pub(super) struct Gathered {
    pub(super) processes: Vec<ProcessRow>,
    pub(super) unresolved: usize,
    pub(super) arguments_unread: usize,
}

pub(super) fn gathered() -> Result<Gathered, CollectError> {
    let entries = processes_running()?;
    let mut buffer = arguments_buffer();
    let mut gathered = Gathered {
        processes: Vec::with_capacity(entries.len()),
        unresolved: 0,
        arguments_unread: 0,
    };

    for entry in entries {
        if entry.pid == 0 || entry.zombie {
            continue;
        }

        let arguments = arguments_of(entry.pid, &mut buffer)
            .ok()
            .and_then(|written| parse_process_arguments(&buffer[..written]));
        let (executable, deleted) = match program_of(executable_of(entry.pid), arguments.as_ref()) {
            Program::Gone => continue,
            Program::At(path) => (Some(path), false),
            Program::Deleted(path) => (Some(path), true),
            Program::Unresolved => {
                gathered.unresolved += 1;
                (None, false)
            }
        };

        let (command_line, command_line_redacted) = match arguments {
            Some(read) if !read.arguments.is_empty() => {
                let clean = redact(&read.arguments);
                (Some(clean.text), clean.redacted)
            }
            Some(_) => (None, false),
            None => {
                gathered.arguments_unread += 1;
                (None, false)
            }
        };

        gathered.processes.push(ProcessRow {
            pid: entry.pid,
            parent: entry.parent,
            uid: entry.real_uid,
            executable,
            executable_deleted: deleted,
            command_line,
            command_line_redacted,
        });
    }

    Ok(gathered)
}
