use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::place::writing_path;
use super::private::create_owner_only;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub inode: u64,
    pub offset: u64,
}

impl Cursor {
    pub fn read(path: &Path) -> Option<Cursor> {
        let text = fs::read_to_string(path).ok()?;
        let mut words = text.split_whitespace();
        Some(Cursor {
            inode: words.next()?.parse().ok()?,
            offset: words.next()?.parse().ok()?,
        })
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        let temporary = writing_path(path);
        {
            let mut file = create_owner_only(&temporary, false)?;
            writeln!(file, "{} {}", self.inode, self.offset)?;
            file.sync_all()?;
        }
        fs::rename(&temporary, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "vigil-cursor-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn what_the_reader_wrote_is_what_the_writer_reads() {
        let path = temporary("round-trip");
        let cursor = Cursor {
            inode: 4_242,
            offset: 1_048_576,
        };

        cursor.write(&path).expect("writes");

        assert_eq!(Cursor::read(&path), Some(cursor));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn no_cursor_and_a_damaged_cursor_are_the_same_answer_and_neither_is_an_error() {
        let missing = temporary("missing");
        assert_eq!(Cursor::read(&missing), None);

        let damaged = temporary("damaged");
        fs::write(&damaged, b"4242").expect("writes");
        assert_eq!(
            Cursor::read(&damaged),
            None,
            "half a cursor is not a cursor"
        );
        let _ = fs::remove_file(&damaged);
    }
}
