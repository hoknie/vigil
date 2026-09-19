use serde_json::Value;

use super::launched::Launched;
use super::moment::epoch_of;
use super::refusal::EsloggerRefusal;

const MOST_ARGUMENTS: usize = 4096;

pub fn parse_eslogger_event(line: &[u8]) -> Result<Launched, EsloggerRefusal> {
    let event: Value = serde_json::from_slice(line).map_err(|_| EsloggerRefusal::NotJson)?;
    let exec = event
        .pointer("/event/exec")
        .filter(|exec| exec.is_object())
        .ok_or(EsloggerRefusal::NotAnExec)?;
    let target = exec
        .get("target")
        .ok_or(EsloggerRefusal::Missing("target"))?;
    let token = target
        .get("audit_token")
        .ok_or(EsloggerRefusal::Missing("audit_token"))?;

    let (seconds, milliseconds) = event
        .get("time")
        .and_then(Value::as_str)
        .and_then(epoch_of)
        .ok_or(EsloggerRefusal::Missing("time"))?;

    let executable = target
        .pointer("/executable/path")
        .and_then(Value::as_str)
        .filter(|path| path.starts_with('/'))
        .ok_or(EsloggerRefusal::Missing("executable path"))?
        .to_string();

    Ok(Launched {
        seconds,
        milliseconds,
        sequence: event
            .get("global_seq_num")
            .or_else(|| event.get("seq_num"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        pid: number(token, "pid").ok_or(EsloggerRefusal::Missing("pid"))?,
        ppid: number(target, "ppid").unwrap_or(0),
        auid: number(token, "auid").ok_or(EsloggerRefusal::Missing("auid"))?,
        uid: number(token, "ruid").unwrap_or(0),
        euid: number(token, "euid").unwrap_or(0),
        executable,
        arguments: exec
            .get("args")
            .and_then(Value::as_array)
            .map(|arguments| {
                arguments
                    .iter()
                    .take(MOST_ARGUMENTS)
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        working_directory: exec
            .pointer("/cwd/path")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn number(object: &Value, field: &str) -> Option<u32> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|number| u32::try_from(number).ok())
}
