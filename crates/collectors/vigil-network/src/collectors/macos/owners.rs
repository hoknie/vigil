use std::collections::BTreeMap;

use vigil_collect::{
    ProcessArguments, Program, arguments_of, executable_of, parse_process_arguments, program_of,
    redact,
};

use crate::parsers::ProcessOwner;

pub(super) fn described(
    holders: &BTreeMap<u64, u32>,
    uids: &BTreeMap<u32, u32>,
    buffer: &mut [u8],
) -> BTreeMap<u64, ProcessOwner> {
    let mut known: BTreeMap<u32, ProcessOwner> = BTreeMap::new();

    holders
        .iter()
        .map(|(handle, pid)| {
            let owner = known
                .entry(*pid)
                .or_insert_with(|| owner_of(*pid, uids.get(pid).copied(), buffer))
                .clone();
            (*handle, owner)
        })
        .collect()
}

fn owner_of(pid: u32, uid: Option<u32>, buffer: &mut [u8]) -> ProcessOwner {
    let arguments: Option<ProcessArguments> = arguments_of(pid, buffer)
        .ok()
        .and_then(|written| parse_process_arguments(&buffer[..written]));

    let (executable, executable_deleted) = match program_of(executable_of(pid), arguments.as_ref())
    {
        Program::At(path) => (Some(path), false),
        Program::Deleted(path) => (Some(path), true),
        Program::Unresolved | Program::Gone => (None, false),
    };

    let (command_line, command_line_redacted) = match arguments {
        Some(read) if !read.arguments.is_empty() => {
            let clean = redact(&read.arguments);
            (Some(clean.text), clean.redacted)
        }
        _ => (None, false),
    };

    ProcessOwner {
        pid: Some(pid),
        executable,
        executable_deleted,
        command_line,
        command_line_redacted,
        uid,
    }
}
