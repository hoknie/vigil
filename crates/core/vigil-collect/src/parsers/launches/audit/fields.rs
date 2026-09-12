pub(super) fn fields(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut at = 0usize;

    while at < bytes.len() {
        while at < bytes.len() && bytes[at] == b' ' {
            at += 1;
        }
        let name_from = at;
        while at < bytes.len() && bytes[at] != b'=' && bytes[at] != b' ' {
            at += 1;
        }
        if at >= bytes.len() || bytes[at] != b'=' {
            while at < bytes.len() && bytes[at] != b' ' {
                at += 1;
            }
            continue;
        }
        let name = &line[name_from..at];
        at += 1;

        let value_from = at;
        if at < bytes.len() && (bytes[at] == b'"' || bytes[at] == b'\'') {
            let quote = bytes[at];
            at += 1;
            while at < bytes.len() && bytes[at] != quote {
                at += 1;
            }
            at = (at + 1).min(bytes.len());
        } else {
            while at < bytes.len() && bytes[at] != b' ' {
                at += 1;
            }
        }
        out.push((name.to_string(), line[value_from..at].to_string()));
    }

    out
}

pub(super) fn value<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, raw)| raw.as_str())
}

pub(super) fn number(fields: &[(String, String)], name: &str) -> Option<u32> {
    value(fields, name)?.parse().ok()
}
