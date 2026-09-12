const FIELD: &str = "CapEff:";

pub fn parse_effective_capabilities(status: &str) -> Option<String> {
    let line = status.lines().find(|line| line.starts_with(FIELD))?;
    let written = line[FIELD.len()..].trim();

    match u64::from_str_radix(written, 16) {
        Ok(mask) => Some(format!("{mask:016x}")),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM_A_CONTAINER: &str = "Name:\tnginx\n\
         Uid:\t0\t0\t0\t0\n\
         CapInh:\t0000000000000000\n\
         CapPrm:\t00000000a80425fb\n\
         CapEff:\t00000000a80425fb\n\
         CapBnd:\t00000000a80425fb\n\
         Seccomp:\t2\n";

    #[test]
    fn the_set_a_process_is_running_with_is_read_and_the_three_beside_it_are_not() {
        assert_eq!(
            parse_effective_capabilities(FROM_A_CONTAINER).as_deref(),
            Some("00000000a80425fb"),
            "what a process may do now is CapEff; the bounding set says what it could be \
             raised to, and reporting that as what it has would call every container \
             privileged"
        );
    }

    #[test]
    fn the_same_set_written_two_ways_by_two_kernels_is_read_as_one_value() {
        let padded = FROM_A_CONTAINER.replace("CapEff:\t00000000a80425fb", "CapEff:\ta80425fb");

        assert_eq!(
            parse_effective_capabilities(&padded),
            parse_effective_capabilities(FROM_A_CONTAINER),
            "a kernel that writes the mask without its leading zeroes has not changed the \
             capabilities of anything, and a reading that differs would be a change on a host \
             that did not move"
        );
    }

    #[test]
    fn a_file_in_a_shape_we_do_not_know_is_not_a_process_with_no_capabilities() {
        assert_eq!(parse_effective_capabilities(""), None);
        assert_eq!(parse_effective_capabilities("CapEff:\tall of them\n"), None);
        assert_eq!(parse_effective_capabilities("Name:\tnginx\n"), None);
    }
}
