use vigil_collect::{Health, name_of_user, processes_running, this_account};

pub(super) fn health() -> Health {
    let entries = match processes_running() {
        Ok(entries) => entries,
        Err(error) => return Health::Unavailable(error.to_string()),
    };

    let me = this_account();
    shown_to(me, entries.iter().any(|entry| entry.effective_uid != me))
}

pub(super) fn shown_to(me: u32, others: bool) -> Health {
    if me == 0 || !others {
        return Health::Ok;
    }

    let named = match name_of_user(me) {
        Some(name) => format!("{name} (uid {me})"),
        None => format!("uid {me}"),
    };
    Health::Degraded(format!(
        "the sockets held by the processes of every account other than {named} are not seen \
         at all, a port they listen on included: macOS lists the open files of a process only \
         to its own account and to root; run as root"
    ))
}
