const LENGTH: usize = 36;

const DASHES_AT: [usize; 4] = [8, 13, 18, 23];

pub fn parse_boot_id(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.len() != LENGTH {
        return None;
    }

    for (at, character) in trimmed.char_indices() {
        let dash = DASHES_AT.contains(&at);
        match dash {
            true if character != '-' => return None,
            false if !character.is_ascii_hexdigit() => return None,
            _ => {}
        }
    }

    Some(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM_A_RUNNING_KERNEL: &str = "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31\n";

    #[test]
    fn the_identifier_a_kernel_writes_is_read_without_the_newline_after_it() {
        assert_eq!(
            parse_boot_id(FROM_A_RUNNING_KERNEL).as_deref(),
            Some("1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31")
        );
    }

    #[test]
    fn a_file_in_a_shape_we_do_not_know_is_not_a_boot_this_agent_names() {
        assert_eq!(parse_boot_id(""), None);
        assert_eq!(parse_boot_id("\n"), None);
        assert_eq!(parse_boot_id("not-a-uuid"), None);
        assert_eq!(
            parse_boot_id("1f0ec2b46c8a4f2b9c0e0b2d4a7f5e31"),
            None,
            "a value that is not the shape the kernel writes would become a key of its own, \
             and every reading that reads it differently would be a reboot"
        );
        assert_eq!(parse_boot_id("1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e3z"), None);
        assert_eq!(
            parse_boot_id("1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31 and more"),
            None
        );
    }
}
