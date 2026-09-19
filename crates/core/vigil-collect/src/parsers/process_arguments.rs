use crate::types::ProcessArguments;

const COUNT_BYTES: usize = 4;

const MOST_ARGUMENTS: usize = 4096;

pub fn parse_process_arguments(bytes: &[u8]) -> Option<ProcessArguments> {
    let count = i32::from_ne_bytes(bytes.get(..COUNT_BYTES)?.try_into().ok()?);
    let count = usize::try_from(count).ok()?.min(MOST_ARGUMENTS);
    let rest = &bytes[COUNT_BYTES..];

    let end = rest.iter().position(|byte| *byte == 0)?;
    let executable = String::from_utf8_lossy(&rest[..end]).into_owned();

    let mut at = end;
    while rest.get(at) == Some(&0) {
        at += 1;
    }

    let mut arguments = Vec::with_capacity(count);
    while arguments.len() < count && at < rest.len() {
        let end = rest[at..]
            .iter()
            .position(|byte| *byte == 0)
            .map_or(rest.len(), |offset| at + offset);
        arguments.push(String::from_utf8_lossy(&rest[at..end]).into_owned());
        at = end + 1;
    }

    Some(ProcessArguments {
        executable,
        arguments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blob(count: i32, executable: &str, padding: usize, rest: &[&str]) -> Vec<u8> {
        let mut bytes = count.to_ne_bytes().to_vec();
        bytes.extend(executable.as_bytes());
        bytes.extend(std::iter::repeat_n(0u8, padding));
        for part in rest {
            bytes.extend(part.as_bytes());
            bytes.push(0);
        }
        bytes
    }

    #[test]
    fn the_program_and_its_arguments_are_read_and_the_environment_after_them_is_not() {
        let bytes = blob(
            3,
            "/usr/bin/mysql",
            7,
            &["mysql", "-psecret", "db", "HOME=/var/root", "TOKEN=abc"],
        );

        let read = parse_process_arguments(&bytes).expect("parses");

        assert_eq!(read.executable, "/usr/bin/mysql");
        assert_eq!(read.arguments, vec!["mysql", "-psecret", "db"]);
        assert!(
            !read
                .arguments
                .iter()
                .any(|argument| argument.contains("TOKEN")),
            "the environment follows the arguments in the same block, and it is the one place \
             on a host where every secret a service was started with is written down"
        );
    }

    #[test]
    fn an_argument_left_empty_is_kept_in_its_place() {
        let bytes = blob(3, "/bin/sh", 1, &["sh", "", "-c"]);

        let read = parse_process_arguments(&bytes).expect("parses");

        assert_eq!(read.arguments, vec!["sh", "", "-c"]);
    }

    #[test]
    fn a_block_cut_before_the_path_of_the_program_ends_is_refused() {
        let mut bytes = 1i32.to_ne_bytes().to_vec();
        bytes.extend(b"/usr/bin/tru");

        assert_eq!(parse_process_arguments(&bytes), None);
        assert_eq!(parse_process_arguments(&[1, 0]), None);
    }

    #[test]
    fn a_block_that_ends_before_its_count_of_arguments_gives_the_arguments_it_holds() {
        let bytes = blob(5, "/bin/sleep", 3, &["sleep", "30"]);

        let read = parse_process_arguments(&bytes).expect("parses");

        assert_eq!(read.arguments, vec!["sleep", "30"]);
    }

    #[test]
    fn a_negative_count_is_not_a_process() {
        assert_eq!(parse_process_arguments(&blob(-1, "/bin/sh", 1, &[])), None);
    }
}
