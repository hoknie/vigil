pub fn one_line(what: &str, value: &str) -> Result<(), String> {
    match value.contains('\n') || value.contains('\r') {
        true => Err(format!(
            "{what} cannot hold a line break: the file it goes into reads one entry per line"
        )),
        false => Ok(()),
    }
}
