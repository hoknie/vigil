use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn commented_lines(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut line = 1;
    let mut at = 0;

    let starts_identifier =
        |at: usize| at > 0 && (chars[at - 1].is_alphanumeric() || chars[at - 1] == '_');

    while at < chars.len() {
        let here = chars[at];
        let next = chars.get(at + 1).copied();

        if here == '\n' {
            line += 1;
            at += 1;
        } else if here == '/' && next == Some('/') {
            found.push(line);
            while at < chars.len() && chars[at] != '\n' {
                at += 1;
            }
        } else if here == '/' && next == Some('*') {
            found.push(line);
            let mut depth = 0;
            while at < chars.len() {
                if chars[at] == '/' && chars.get(at + 1) == Some(&'*') {
                    depth += 1;
                    at += 2;
                } else if chars[at] == '*' && chars.get(at + 1) == Some(&'/') {
                    depth -= 1;
                    at += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if chars[at] == '\n' {
                        line += 1;
                    }
                    at += 1;
                }
            }
        } else if let Some(hashes) = raw_string_at(&chars, at).filter(|_| !starts_identifier(at)) {
            at += hashes.1;
            let closing: String = std::iter::once('"')
                .chain(std::iter::repeat_n('#', hashes.0))
                .collect();
            let closing: Vec<char> = closing.chars().collect();
            while at < chars.len() && !chars[at..].starts_with(&closing) {
                if chars[at] == '\n' {
                    line += 1;
                }
                at += 1;
            }
            at += closing.len();
        } else if here == '"' {
            at += 1;
            while at < chars.len() && chars[at] != '"' {
                if chars[at] == '\\' {
                    at += 1;
                }
                if chars.get(at) == Some(&'\n') {
                    line += 1;
                }
                at += 1;
            }
            at += 1;
        } else if here == '\'' {
            match (next, chars.get(at + 2)) {
                (Some('\\'), _) => {
                    at += 2;
                    while at < chars.len() && chars[at] != '\'' {
                        at += 1;
                    }
                    at += 1;
                }
                (Some(_), Some('\'')) => at += 3,
                _ => at += 1,
            }
        } else {
            at += 1;
        }
    }

    found.dedup();
    found
}

fn raw_string_at(chars: &[char], at: usize) -> Option<(usize, usize)> {
    let mut cursor = at;
    if chars.get(cursor) == Some(&'b') {
        cursor += 1;
    }
    if chars.get(cursor) != Some(&'r') {
        return None;
    }
    cursor += 1;
    let mut hashes = 0;
    while chars.get(cursor) == Some(&'#') {
        hashes += 1;
        cursor += 1;
    }
    (chars.get(cursor) == Some(&'"')).then_some((hashes, cursor + 1 - at))
}

fn rust_files(under: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(under) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == "target") {
            continue;
        }
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            into.push(path);
        }
    }
}

#[test]
fn the_lexer_finds_every_kind_of_comment_and_nothing_inside_a_literal() {
    for clean in [
        "let url = \"https://example.com/a//b\";",
        "let raw = r#\"// not a comment\"#;",
        "let slash = '/';",
        "fn held<'a>(text: &'a str) -> &'a str { text }",
        "let bytes = b\"//\";",
        "let quote = '\\'';",
        "let raw = br\"/* */\";",
        "let escaped = \"a \\\" // still a string\";",
    ] {
        assert!(commented_lines(clean).is_empty(), "{clean}");
    }

    for dirty in [
        "let x = 1; // why",
        "/// what this does\nfn x() {}",
        "//! the crate",
        "let x = /* inline */ 1;",
        "let a = 'a'; // after a character",
    ] {
        assert!(!commented_lines(dirty).is_empty(), "{dirty}");
    }
    assert_eq!(
        commented_lines("let a = 1;\n\nlet b = 2; // here\n"),
        vec![3]
    );
}

#[test]
fn no_rust_file_carries_a_comment() {
    let mut files = Vec::new();
    for top in ["crates", "bin"] {
        rust_files(&root().join(top), &mut files);
    }
    assert!(files.len() > 100, "only {} file(s) were read", files.len());

    let mut found: Vec<String> = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("readable");
        for line in commented_lines(&text) {
            found.push(format!(
                "{}:{line}",
                file.strip_prefix(root()).unwrap_or(file).display()
            ));
        }
    }

    assert!(
        found.is_empty(),
        "a .rs file holds no comment: a decision is held by a test named after it, and prose for \
         a reader of the code goes in the crate's README. Found at:\n{}",
        found.join("\n")
    );
}

fn comment_starts(text: &str) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut heredoc: Option<(String, bool)> = None;

    for (index, line) in text.split('\n').enumerate() {
        if let Some((closing, indented)) = &heredoc {
            let ends = match indented {
                true => line.trim_start() == closing,
                false => line == closing,
            };
            if ends {
                heredoc = None;
            }
            continue;
        }

        let chars: Vec<char> = line.chars().collect();
        let mut quote: Option<char> = None;
        let mut at = 0;
        while at < chars.len() {
            let here = chars[at];
            match quote {
                Some('\'') if here == '\'' => quote = None,
                Some('"') if here == '\\' => at += 1,
                Some('"') if here == '"' => quote = None,
                Some(_) => {}
                None if here == '\\' => at += 1,
                None if here == '\'' || here == '"' => quote = Some(here),
                None if here == '#' && (at == 0 || chars[at - 1].is_whitespace()) => {
                    found.push((index + 1, at));
                    break;
                }
                None => {}
            }
            at += 1;
        }

        if let Some(opened) = heredoc_opened(line) {
            heredoc = Some(opened);
        }
    }

    found
}

fn prose_lines(text: &str, just: bool) -> Vec<usize> {
    let lines: Vec<&str> = text.split('\n').collect();

    comment_starts(text)
        .into_iter()
        .filter(|(number, at)| {
            let line = lines[number - 1];
            let held = line.trim_start();
            let is_comment_line = line.chars().take(*at).all(char::is_whitespace);

            if *number == 1 && held.starts_with("#!") {
                return false;
            }
            if is_comment_line
                && (held.starts_with("# shellcheck") || held.starts_with("# syntax="))
                && !held[1..].contains('#')
            {
                return false;
            }
            !(just && is_comment_line && describes_what_follows(lines.get(*number).copied()))
        })
        .map(|(number, _)| number)
        .collect()
}

fn describes_what_follows(next: Option<&str>) -> bool {
    let Some(next) = next else {
        return false;
    };
    if next.starts_with([' ', '\t']) || next.trim().is_empty() {
        return false;
    }
    if next.starts_with('[') || next.starts_with("mod ") {
        return true;
    }
    let name: String = next
        .chars()
        .take_while(|character| {
            character.is_alphanumeric() || *character == '-' || *character == '_'
        })
        .collect();
    let rest = &next[name.len()..];
    !name.is_empty()
        && rest.contains(':')
        && !rest.trim_start().starts_with(":=")
        && !rest.contains(":=")
}

fn heredoc_opened(line: &str) -> Option<(String, bool)> {
    let after = line.split("<<").nth(1)?;
    if after.starts_with('<') {
        return None;
    }
    let indented = after.starts_with('-');
    let word: String = after
        .trim_start_matches('-')
        .trim_start()
        .trim_start_matches(['\'', '"'])
        .chars()
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .collect();
    (!word.is_empty()).then_some((word, indented))
}

fn build_files() -> Vec<(PathBuf, bool)> {
    let root = root();
    let mut files: Vec<(PathBuf, bool)> = vec![
        (root.join("Cargo.toml"), false),
        (root.join("justfile"), true),
    ];
    for members in [
        "crates/core",
        "crates/collectors",
        "crates/consumers",
        "bin",
    ] {
        for entry in fs::read_dir(root.join(members))
            .into_iter()
            .flatten()
            .flatten()
        {
            let manifest = entry.path().join("Cargo.toml");
            if manifest.exists() {
                files.push((manifest, false));
            }
        }
    }
    let mut walked = Vec::new();
    for top in [
        "env/justice",
        "env/docker",
        "env/scripts",
        "packaging",
        ".github/workflows",
    ] {
        everything(&root.join(top), &mut walked);
    }
    for path in walked {
        let relative = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();
        if relative.starts_with("packaging/systemd")
            || relative.starts_with("packaging/audit")
            || path.extension().is_some_and(|extension| extension == "md")
        {
            continue;
        }
        let just = path
            .extension()
            .is_some_and(|extension| extension == "just");
        files.push((path, just));
    }
    files
}

fn everything(under: &Path, into: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(under).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            everything(&path, into);
        } else {
            into.push(path);
        }
    }
}

#[test]
fn the_build_file_reader_allows_what_a_tool_reads_and_nothing_else() {
    let script = "#!/usr/bin/env bash\n\
                  # shellcheck disable=SC2086\n\
                  echo \"a # b\"\n\
                  count=$#\n\
                  length=${#name}\n\
                  cat <<'EOF'\n\
                  # Changelog\n\
                  EOF\n";
    assert!(
        prose_lines(script, false).is_empty(),
        "{:?}",
        prose_lines(script, false)
    );

    assert_eq!(
        prose_lines(
            "# shellcheck disable=SC2086  # the words split on purpose\n",
            false
        ),
        vec![1]
    );
    assert_eq!(prose_lines("set -e\nrun  # why it runs\n", false), vec![2]);
    assert_eq!(
        prose_lines("# explains the next line\nrun\n", false),
        vec![1]
    );

    let recipes = "# What the recipe does\n[group('build')]\nbuild:\n    cargo build\n\n# Another one\nrun what:\n    true\n";
    assert!(
        prose_lines(recipes, true).is_empty(),
        "{:?}",
        prose_lines(recipes, true)
    );
    assert_eq!(
        prose_lines("# a note with nothing under it\n\nbuild:\n    true\n", true),
        vec![1]
    );
    assert!(
        prose_lines(
            "# Print one screen\ncapture screen='summary':\n    true\n",
            true
        )
        .is_empty(),
        "a recipe with a parameter is still a recipe"
    );
    let awk = "awk '\n    /^## / { print }\n' file\necho \"\\\"${tag#v}\\\"\"\ntext=\"$(section \"## [x]\")\"\n";
    assert!(
        prose_lines(awk, false).is_empty(),
        "a # inside a quoted program that spans lines, an escaped quote or a nested substitution \
         is text: {:?}",
        prose_lines(awk, false)
    );
}

#[test]
fn no_build_or_packaging_file_carries_prose_in_a_comment() {
    let files = build_files();
    assert!(files.len() > 30, "only {} file(s) were read", files.len());

    let mut found = Vec::new();
    for (file, just) in &files {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        for line in prose_lines(&text, *just) {
            found.push(format!(
                "{}:{line}",
                file.strip_prefix(root()).unwrap_or(file).display()
            ));
        }
    }

    assert!(
        found.is_empty(),
        "a build or packaging file carries no prose beside its lines: a name, a recipe or a \
         test says it. What stays is a shebang, a directive a tool reads, the line just prints \
         as a recipe's description and text a script prints. Found at:\n{}",
        found.join("\n")
    );
}
