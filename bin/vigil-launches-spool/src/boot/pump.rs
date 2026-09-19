use std::io::{self, BufRead, BufReader, Read};

use vigil_launches::{
    EsloggerRefusal, SpoolWriter, audit_records, parse_eslogger_event, somebody_launched,
};

const LONGEST_LINE: usize = 8 * 1024 * 1024;

const WRITTEN_AT_ONCE: usize = 64 * 1024;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pumped {
    pub written: u64,
    pub nobody: u64,
    pub other_events: u64,
    pub unreadable: u64,
    pub unwritten: u64,
    pub writes: u64,
}

impl Pumped {
    pub fn events(&self) -> u64 {
        self.written + self.nobody + self.other_events + self.unwritten
    }
}

pub fn pump<R: Read>(
    printed: &mut BufReader<R>,
    spool: &mut SpoolWriter,
    keep_arguments: bool,
    unwritable: &mut Option<String>,
) -> Pumped {
    let mut pumped = Pumped::default();
    let mut line: Vec<u8> = Vec::new();
    let mut pending: Vec<u8> = Vec::new();
    let mut waiting = 0u64;

    loop {
        if !pending.is_empty() && (pending.len() >= WRITTEN_AT_ONCE || printed.buffer().is_empty())
        {
            written(spool, &mut pending, &mut waiting, &mut pumped, unwritable);
        }
        line.clear();
        match printed.read_until(b'\n', &mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
        if line.len() > LONGEST_LINE {
            pumped.unreadable += 1;
            continue;
        }

        let launched = match parse_eslogger_event(line.trim_ascii()) {
            Ok(launched) => launched,
            Err(EsloggerRefusal::NotAnExec) => {
                pumped.other_events += 1;
                continue;
            }
            Err(_) => {
                pumped.unreadable += 1;
                continue;
            }
        };
        if !somebody_launched(&launched) {
            pumped.nobody += 1;
            continue;
        }

        pending.extend_from_slice(audit_records(&launched, keep_arguments).as_bytes());
        waiting += 1;
    }

    written(spool, &mut pending, &mut waiting, &mut pumped, unwritable);
    pumped
}

fn written(
    spool: &mut SpoolWriter,
    pending: &mut Vec<u8>,
    waiting: &mut u64,
    pumped: &mut Pumped,
    unwritable: &mut Option<String>,
) {
    if pending.is_empty() {
        return;
    }
    match spool.write(pending) {
        Ok(()) => pumped.written += *waiting,
        Err(error) => {
            pumped.unwritten += *waiting;
            if unwritable.is_none() {
                eprintln!(
                    "vigil-launches-spool: cannot write the spool: {error}. Reading continues, \
                     so a full disk cannot stop eslogger"
                );
                *unwritable = Some(error.to_string());
            }
        }
    }
    pumped.writes += 1;
    pending.clear();
    *waiting = 0;
}
