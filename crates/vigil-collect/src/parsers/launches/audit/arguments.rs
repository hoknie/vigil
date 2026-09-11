use std::collections::BTreeMap;

use super::text::text;

pub(super) fn arguments(execve: &[(String, String)]) -> (Vec<String>, bool) {
    let mut pieces: BTreeMap<usize, BTreeMap<usize, String>> = BTreeMap::new();
    let mut lossy = false;

    for (name, raw) in execve {
        let Some(rest) = name.strip_prefix('a') else {
            continue;
        };
        let (index, piece) = match rest.split_once('[') {
            Some((index, piece)) => (index, piece.trim_end_matches(']')),
            None => (rest, "0"),
        };
        let (Ok(index), Ok(piece)) = (index.parse::<usize>(), piece.parse::<usize>()) else {
            continue;
        };

        let (value, was_lossy) = text(raw);
        lossy |= was_lossy;
        if let Some(value) = value {
            pieces.entry(index).or_default().insert(piece, value);
        }
    }

    let joined = pieces
        .into_values()
        .map(|parts| parts.into_values().collect::<String>())
        .collect();
    (joined, lossy)
}
