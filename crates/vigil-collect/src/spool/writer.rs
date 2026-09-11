use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::parsers::record_is_read;

use super::cursor::Cursor;
use super::marker;
use super::place::{cursor_path, writing_path};
use super::private::{create_owner_only, owner_only_directory};

const MAX_RECORD_BYTES: usize = 64 * 1024;

const COPY_CHUNK: usize = 64 * 1024;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    pub kept: u64,
    pub skipped: u64,
    pub oversized: u64,
    pub compactions: u64,
    pub dropped: u64,
}

pub struct SpoolWriter {
    path: PathBuf,
    ceiling: u64,
    file: File,
    bytes: u64,
    pending: Vec<u8>,
    resyncing: bool,
    report: Report,
}

impl SpoolWriter {
    pub fn open(path: impl Into<PathBuf>, ceiling: u64) -> io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
            owner_only_directory(parent)?;
        }

        let file = create_owner_only(&path, true)?;
        let bytes = file.metadata()?.len();
        let mut writer = SpoolWriter {
            path,
            ceiling,
            file,
            bytes,
            pending: Vec::new(),
            resyncing: false,
            report: Report::default(),
        };
        writer.close_a_torn_record()?;
        Ok(writer)
    }

    pub fn write(&mut self, chunk: &[u8]) -> io::Result<()> {
        self.pending.extend_from_slice(chunk);

        let mut wanted: Vec<u8> = Vec::new();
        let mut kept = 0u64;
        let mut skipped = 0u64;
        let mut resyncing = self.resyncing;
        let mut consumed = 0usize;

        while let Some(at) = self.pending[consumed..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let line = &self.pending[consumed..consumed + at + 1];
            consumed += at + 1;

            if resyncing {
                resyncing = false;
                continue;
            }
            if record_is_read(line) {
                kept += 1;
                wanted.extend_from_slice(line);
            } else {
                skipped += 1;
            }
        }

        self.pending.drain(..consumed);
        if self.pending.len() > MAX_RECORD_BYTES {
            self.pending.clear();
            resyncing = true;
            self.report.oversized += 1;
        }

        self.resyncing = resyncing;
        self.report.kept += kept;
        self.report.skipped += skipped;

        if !wanted.is_empty() {
            self.append(&wanted)?;
        }
        Ok(())
    }

    pub fn report(&self) -> Report {
        self.report
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.file.write_all(bytes)?;
        self.file.sync_data()?;
        self.bytes += bytes.len() as u64;

        if self.bytes > self.ceiling {
            self.compact()?;
        }
        Ok(())
    }

    fn compact(&mut self) -> io::Result<()> {
        let inode = inode_of(&self.file)?;
        let read_to = Cursor::read(&cursor_path(&self.path))
            .filter(|cursor| cursor.inode == inode && cursor.offset <= self.bytes)
            .map_or(0, |cursor| cursor.offset);

        let mut start = read_to;
        let mut dropped = 0u64;
        if self.bytes - start > self.ceiling {
            let wanted = self.bytes - self.ceiling / 2;
            start = next_record_at_or_after(&self.path, wanted)?;
            dropped = start.saturating_sub(read_to);
        }
        if start == 0 {
            return Ok(());
        }

        let temporary = writing_path(&self.path);
        {
            let mut source = File::open(&self.path)?;
            source.seek(SeekFrom::Start(start))?;
            let mut target = create_owner_only(&temporary, false)?;
            if dropped > 0 {
                target.write_all(marker::dropped_line(dropped, self.ceiling).as_bytes())?;
            }
            copy(&mut source, &mut target)?;
            target.sync_all()?;
        }
        fs::rename(&temporary, &self.path)?;

        let _ = fs::remove_file(cursor_path(&self.path));

        self.file = create_owner_only(&self.path, true)?;
        self.bytes = self.file.metadata()?.len();
        self.report.compactions += 1;
        self.report.dropped += dropped;
        Ok(())
    }

    fn close_a_torn_record(&mut self) -> io::Result<()> {
        if self.bytes == 0 {
            return Ok(());
        }
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::End(-1))?;
        let mut last = [0u8; 1];
        file.read_exact(&mut last)?;
        if last[0] != b'\n' {
            self.file.write_all(b"\n")?;
            self.file.sync_data()?;
            self.bytes += 1;
        }
        Ok(())
    }
}

fn next_record_at_or_after(path: &Path, wanted: u64) -> io::Result<u64> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(wanted))?;

    let mut at = wanted;
    let mut buffer = [0u8; COPY_CHUNK];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            return Ok(at);
        }
        if let Some(newline) = buffer[..read].iter().position(|byte| *byte == b'\n') {
            return Ok(at + newline as u64 + 1);
        }
        at += read as u64;
    }
}

fn copy(source: &mut File, target: &mut File) -> io::Result<u64> {
    let mut buffer = vec![0u8; COPY_CHUNK];
    let mut moved = 0u64;
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            return Ok(moved);
        }
        target.write_all(&buffer[..read])?;
        moved += read as u64;
    }
}

#[cfg(unix)]
fn inode_of(file: &File) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    Ok(file.metadata()?.ino())
}

#[cfg(not(unix))]
fn inode_of(_file: &File) -> io::Result<u64> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYSCALL: &str = concat!(
        r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 auid=1000 uid=1000 comm="id" exe="/usr/bin/id" key="vigil_exec""#,
        "\n"
    );
    const EXECVE: &str = "type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0=\"id\"\n";
    const PROCTITLE: &str =
        "type=PROCTITLE msg=audit(1757419203.412:3421): proctitle=2F7573722F62696E2F6964\n";
    const LOGIN: &str =
        "type=USER_LOGIN msg=audit(1757419203.400:3400): pid=1 uid=0 auid=1000 res=success\n";

    fn workspace(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join("vigil-spool-test");
        fs::create_dir_all(&directory).expect("temp dir");
        let path = directory.join(format!("{}-{name}", std::process::id()));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(cursor_path(&path));
        path
    }

    fn contents(path: &Path) -> String {
        fs::read_to_string(path).expect("readable")
    }

    #[test]
    fn what_the_collector_reads_is_written_and_what_it_never_reads_is_not() {
        let path = workspace("filter");
        let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");

        spool
            .write(format!("{LOGIN}{SYSCALL}{PROCTITLE}{EXECVE}").as_bytes())
            .expect("writes");

        let written = contents(&path);
        assert!(written.contains("type=SYSCALL"), "{written}");
        assert!(written.contains("type=EXECVE"), "{written}");
        assert!(!written.contains("USER_LOGIN"), "{written}");
        assert!(!written.contains("PROCTITLE"), "{written}");
        assert_eq!(spool.report().kept, 2);
        assert_eq!(spool.report().skipped, 2);
    }

    #[test]
    fn a_record_split_across_two_reads_is_written_once_and_whole() {
        let path = workspace("split");
        let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");
        let (head, tail) = SYSCALL.split_at(40);

        spool.write(head.as_bytes()).expect("writes");
        assert_eq!(contents(&path), "", "half a record is not a record");
        spool.write(tail.as_bytes()).expect("writes");

        assert_eq!(contents(&path), SYSCALL);
    }

    #[test]
    fn a_line_longer_than_any_record_is_given_up_on_rather_than_buffered_for_ever() {
        let path = workspace("oversized");
        let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");

        spool
            .write(&vec![b'x'; MAX_RECORD_BYTES + 10])
            .expect("writes");
        spool
            .write(format!("xxxxx\n{SYSCALL}").as_bytes())
            .expect("writes");

        assert_eq!(spool.report().oversized, 1);
        assert_eq!(
            contents(&path),
            SYSCALL,
            "the record after the damage must survive"
        );
    }

    #[test]
    fn the_ceiling_drops_the_oldest_and_says_so_when_nothing_is_reading() {
        let path = workspace("ceiling");
        let ceiling = 8 * 1024;
        let mut spool = SpoolWriter::open(path.clone(), ceiling).expect("opens");

        for _ in 0..200 {
            spool.write(SYSCALL.as_bytes()).expect("writes");
        }

        assert!(
            spool.bytes() <= ceiling,
            "the spool grew past its ceiling: {} bytes",
            spool.bytes()
        );
        assert!(spool.report().dropped > 0, "a loss must be counted");
        let written = contents(&path);
        let first = written.lines().next().expect("a first line");
        assert!(
            marker::dropped_note(first.as_bytes()).is_some(),
            "the loss must be announced at the head of the file: {first}"
        );
        for line in written.lines().skip(1) {
            assert!(line.starts_with("type="), "cut mid-record: {line}");
        }
    }

    #[test]
    fn what_the_reader_has_already_consumed_is_reclaimed_without_anything_being_lost() {
        let path = workspace("consumed");
        let ceiling = 8 * 1024;
        let mut spool = SpoolWriter::open(path.clone(), ceiling).expect("opens");
        for _ in 0..30 {
            spool.write(SYSCALL.as_bytes()).expect("writes");
        }
        assert!(spool.bytes() < ceiling, "the first batch must fit");

        let inode = inode_of(&spool.file).expect("stat");
        Cursor {
            inode,
            offset: spool.bytes(),
        }
        .write(&cursor_path(&path))
        .expect("cursor");

        for _ in 0..30 {
            spool.write(SYSCALL.as_bytes()).expect("writes");
        }

        assert_eq!(spool.report().compactions, 1, "it should have compacted");
        assert_eq!(spool.report().dropped, 0, "nothing was lost");
        assert!(
            !cursor_path(&path).exists(),
            "the cursor of the file that has just been replaced must not survive it — inode \
             numbers are reused, and a stale cursor that passes the inode test is a silent loss"
        );
        let written = contents(&path);
        assert!(
            marker::dropped_note(written.lines().next().unwrap_or("").as_bytes()).is_none(),
            "a compaction that lost nothing must not claim it did: {written}"
        );
        assert_eq!(written.lines().count(), 30, "only what was unread is left");
    }

    #[test]
    fn a_spool_torn_by_a_power_cut_costs_one_record_and_not_the_one_behind_it() {
        let path = workspace("torn");
        fs::write(&path, format!("{SYSCALL}type=SYSCALL msg=aud")).expect("writes");

        let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");
        spool.write(EXECVE.as_bytes()).expect("writes");

        let written = contents(&path);
        let lines: Vec<&str> = written.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[1], "type=SYSCALL msg=aud",
            "the torn record stays torn"
        );
        assert_eq!(
            lines[2],
            EXECVE.trim_end(),
            "and the record after it is whole"
        );
    }

    #[test]
    fn an_empty_stream_leaves_an_empty_spool_and_not_a_missing_one() {
        let path = workspace("empty");

        let spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");

        assert_eq!(spool.bytes(), 0);
        assert!(path.exists());
    }
}
