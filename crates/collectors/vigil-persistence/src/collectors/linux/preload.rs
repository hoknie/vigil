use std::fs;

use crate::parsers::PreloadFile;
use vigil_collect::{absent, hex, sha256};

use super::PRELOAD;

pub(super) fn read_preload() -> PreloadFile {
    let mut preload = PreloadFile {
        path: PRELOAD.to_string(),
        present: false,
        readable: true,
        entries: Vec::new(),
        digest: None,
    };

    match fs::read_to_string(PRELOAD) {
        Ok(text) => {
            preload.present = true;
            preload.digest = Some(hex(&sha256(text.as_bytes())));
            preload.entries = text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_string)
                .collect();
        }
        Err(error) if absent(&error) => {}
        Err(_) => {
            preload.present = true;
            preload.readable = false;
        }
    }

    preload
}
