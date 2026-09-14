use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::parsers::record_is_read;
use crate::spool::cursor::Cursor;
use crate::spool::marker;
use crate::spool::place::{cursor_path, writing_path};
use crate::spool::private::{create_owner_only, owner_only_directory};

use super::on_disk::{copy, inode_of, next_record_at_or_after};
use super::{Report, SpoolWriter};

pub(super) const MAX_RECORD_BYTES: usize = 64 * 1024;

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
