use vigil_collect::{
    ProcessArguments, Program, arguments_buffer, arguments_of, executable_of,
    parse_process_arguments, parse_process_table, process_entry, processes_running, program_of,
};

pub fn running(executable: &str, uid: u32) -> Result<Vec<u32>, String> {
    let entries = processes_running().map_err(|error| error.to_string())?;

    let mut pids: Vec<u32> = entries
        .iter()
        .filter(|entry| entry.pid != 0 && !entry.zombie && entry.real_uid == uid)
        .map(|entry| entry.pid)
        .filter(|pid| still_running(*pid, executable, None))
        .collect();
    pids.sort_unstable();
    Ok(pids)
}

pub fn still_running(pid: u32, executable: &str, uid: Option<u32>) -> bool {
    if let Some(uid) = uid {
        let Some(entry) = process_entry(pid)
            .ok()
            .and_then(|bytes| parse_process_table(&bytes))
            .and_then(|entries| entries.into_iter().next())
        else {
            return false;
        };
        if entry.real_uid != uid || entry.zombie {
            return false;
        }
    }

    let answer = executable_of(pid);
    let arguments = match &answer {
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => started_as(pid),
        _ => None,
    };

    match program_of(answer, arguments.as_ref()) {
        Program::At(path) | Program::Deleted(path) => path == executable,
        Program::Unresolved | Program::Gone => false,
    }
}

fn started_as(pid: u32) -> Option<ProcessArguments> {
    let mut buffer = arguments_buffer();
    let written = arguments_of(pid, &mut buffer).ok()?;

    parse_process_arguments(&buffer[..written])
}
