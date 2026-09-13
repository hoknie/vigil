use super::session::Session;

pub fn merge_sessions(mut records: Vec<Session>) -> Vec<Session> {
    records.sort_by_key(|record| {
        (
            record.line.is_empty(),
            record.who(),
            record.line.clone(),
            record.id.clone(),
        )
    });

    let mut merged: Vec<Session> = Vec::new();
    for record in records {
        match merged.iter_mut().find(|held| same_session(held, &record)) {
            Some(held) => fold(held, record),
            None => merged.push(record),
        }
    }

    merged.sort_by_key(Session::key);
    merged
}

fn same_session(held: &Session, record: &Session) -> bool {
    if held.who() != record.who() {
        return false;
    }
    held.place() == record.place() || (held.pid != 0 && held.pid == record.pid)
}

fn fold(held: &mut Session, record: Session) {
    take(&mut held.user, record.user);
    take(&mut held.line, record.line);
    take(&mut held.from, record.from);
    take(&mut held.id, record.id);
    take(&mut held.service, record.service);
    take(&mut held.kind, record.kind);
    take(&mut held.class, record.class);
    take(&mut held.state, record.state);
    held.remote = held.remote || record.remote;
    held.uid = held.uid.or(record.uid);
    if held.pid == 0 {
        held.pid = record.pid;
    }
    held.sources.extend(record.sources);
}

fn take(held: &mut String, offered: String) {
    if held.is_empty() {
        *held = offered;
    }
}
