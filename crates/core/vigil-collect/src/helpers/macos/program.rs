use std::io;

use crate::types::{ProcessArguments, Program};

pub fn program_of(answer: io::Result<String>, arguments: Option<&ProcessArguments>) -> Program {
    match answer {
        Ok(path) => Program::At(path),
        Err(error) if error.raw_os_error() == Some(libc::ESRCH) => Program::Gone,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => {
            match arguments.map(|read| read.executable.as_str()) {
                Some(path) if path.starts_with('/') => Program::Deleted(path.to_string()),
                _ => Program::Unresolved,
            }
        }
        Err(_) => Program::Unresolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started_as(path: &str) -> ProcessArguments {
        ProcessArguments {
            executable: path.to_string(),
            arguments: vec![path.to_string()],
        }
    }

    #[test]
    fn a_program_whose_file_is_there_is_named_by_the_path_of_its_file() {
        assert_eq!(
            program_of(Ok("/usr/sbin/sshd".into()), None),
            Program::At("/usr/sbin/sshd".into())
        );
    }

    #[test]
    fn a_program_whose_file_was_deleted_is_named_by_the_path_it_was_started_from() {
        let answer = Err(io::Error::from_raw_os_error(libc::ENOENT));

        assert_eq!(
            program_of(answer, Some(&started_as("/private/tmp/.x/nc"))),
            Program::Deleted("/private/tmp/.x/nc".into()),
            "macOS will not name the file of a process once the file is gone, and the path the \
             process was started from is the one the kernel kept"
        );
    }

    #[test]
    fn a_deleted_program_started_by_a_relative_path_is_not_given_a_path_it_never_had() {
        let answer = Err(io::Error::from_raw_os_error(libc::ENOENT));

        assert_eq!(
            program_of(answer, Some(&started_as("./nc"))),
            Program::Unresolved
        );
        assert_eq!(
            program_of(Err(io::Error::from_raw_os_error(libc::ENOENT)), None),
            Program::Unresolved,
            "a deleted program of another account, whose arguments are not shown, is a program \
             this reading could not name, and it is counted as one"
        );
    }

    #[test]
    fn a_process_that_exited_between_the_table_and_the_question_is_not_a_program() {
        let answer = Err(io::Error::from_raw_os_error(libc::ESRCH));

        assert_eq!(program_of(answer, None), Program::Gone);
    }

    #[test]
    fn a_refusal_is_a_program_this_reading_could_not_name() {
        let answer = Err(io::Error::from_raw_os_error(libc::EPERM));

        assert_eq!(program_of(answer, None), Program::Unresolved);
    }
}
