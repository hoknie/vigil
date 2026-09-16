pub type Moment = (u64, u64, u64);

pub fn moment(id: &str) -> Option<Moment> {
    let (time, serial) = id.split_once(':')?;
    let (seconds, milliseconds) = time.split_once('.')?;
    Some((
        seconds.parse().ok()?,
        milliseconds.parse().ok()?,
        serial.parse().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_audit_id_is_read_as_the_second_the_millisecond_and_the_serial_the_kernel_wrote() {
        assert_eq!(
            moment("1757419203.412:3421"),
            Some((1_757_419_203, 412, 3421))
        );
        for broken in [
            "",
            "1757419203.412",
            "1757419203:3421",
            "a.412:3421",
            "1.2:x",
        ] {
            assert_eq!(
                moment(broken),
                None,
                "{broken:?}: an id this build cannot read orders nowhere, rather than at zero"
            );
        }
    }
}
