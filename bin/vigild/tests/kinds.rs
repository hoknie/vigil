use std::fs;
use std::path::{Path, PathBuf};

const KINDS: &[&str] = &[
    "ports",
    "collectors",
    "parsers",
    "rules",
    "services",
    "stores",
    "sinks",
    "formats",
    "screens",
    "views",
    "modules",
    "types",
    "helpers",
    "loops",
    "fixture",
    "adapters",
];

const CONSTRUCT_FOLDERS: &[&str] = &[
    "structs", "enums", "traits", "impls", "utils", "common", "misc", "core",
];

const CATCH_ALL_FILES: &[&str] = &["misc.rs", "common.rs", "utils.rs", "util.rs"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for members in [
        "crates/core",
        "crates/collectors",
        "crates/consumers",
        "bin",
    ] {
        for member in fs::read_dir(root().join(members))
            .into_iter()
            .flatten()
            .flatten()
        {
            let source = member.path().join("src");
            if source.is_dir() {
                found.push(source);
            }
        }
    }
    found
}

fn walked(under: &Path, into: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(under).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            into.push(path.clone());
            walked(&path, into);
        } else {
            into.push(path);
        }
    }
}

fn every_path() -> Vec<(PathBuf, PathBuf)> {
    let mut found = Vec::new();
    for source in sources() {
        let mut paths = Vec::new();
        walked(&source, &mut paths);
        found.extend(paths.into_iter().map(|path| (source.clone(), path)));
    }
    found
}

fn shown(path: &Path) -> String {
    path.strip_prefix(root())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn declaring_only(text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut brackets = 0usize;
    let mut in_use = false;

    for (index, line) in text.lines().enumerate() {
        let held = line.trim();
        if brackets > 0 {
            brackets = balance(brackets, held);
            continue;
        }
        if in_use {
            in_use = !held.ends_with(';');
            continue;
        }
        if held.is_empty() {
            continue;
        }
        if held.starts_with("#[") || held.starts_with("#![") {
            brackets = balance(0, held);
            continue;
        }
        let statement = visibility_dropped(held);
        if statement.starts_with("mod ") && statement.ends_with(';') {
            continue;
        }
        if statement.starts_with("use ") {
            in_use = !statement.ends_with(';');
            continue;
        }
        found.push(index + 1);
    }

    found
}

fn visibility_dropped(held: &str) -> &str {
    if let Some(rest) = held.strip_prefix("pub(")
        && let Some((_, after)) = rest.split_once(") ")
    {
        return after;
    }
    held.strip_prefix("pub ").unwrap_or(held)
}

fn balance(open: usize, line: &str) -> usize {
    let opened = line.matches('[').count();
    let closed = line.matches(']').count();
    (open + opened).saturating_sub(closed)
}

#[test]
fn the_root_reader_knows_a_declaration_from_code() {
    let declaring = "#![cfg(target_os = \"linux\")]\n\n#[cfg_attr(\n    not(target_os = \"macos\"),\n    allow(dead_code)\n)]\nmod parsers;\npub(crate) mod rules;\npub(in crate::ui) mod menu;\npub(in crate::views) use path::drawn;\n\npub use parsers::{\n    One,\n    Two,\n};\n";
    assert!(
        declaring_only(declaring).is_empty(),
        "{:?}",
        declaring_only(declaring)
    );

    assert_eq!(
        declaring_only("mod a;\n\nconst LIMIT: usize = 3;\n"),
        vec![3]
    );
    assert!(
        declaring_only("#[cfg(test)]\nmod tests {\n    #[test]\n    fn x() {}\n}\n").contains(&2),
        "a test module with a body is code in the root: the tests belong in tests.rs"
    );
    assert_eq!(declaring_only("pub struct Held;\n"), vec![1]);
}

#[test]
fn no_folder_or_file_is_named_after_a_rust_construct_or_a_catch_all() {
    let found: Vec<String> = every_path()
        .into_iter()
        .filter(|(_, path)| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            match path.is_dir() {
                true => CONSTRUCT_FOLDERS.contains(&name),
                false => CATCH_ALL_FILES.contains(&name),
            }
        })
        .map(|(_, path)| shown(&path))
        .collect();

    assert!(
        found.is_empty(),
        "a folder is named after the kind of thing it holds and a file after the thing: a \
         folder of structs or a file of utils names neither. Found:\n{}",
        found.join("\n")
    );
}

#[test]
fn a_kind_folder_holds_only_its_own_kind() {
    let found: Vec<String> = every_path()
        .into_iter()
        .filter(|(_, path)| path.is_dir())
        .filter(|(source, path)| {
            let kinds: Vec<&str> = path
                .strip_prefix(source)
                .unwrap_or(path)
                .components()
                .filter_map(|part| part.as_os_str().to_str())
                .filter(|part| KINDS.contains(part))
                .collect();
            kinds.windows(2).any(|pair| pair[0] != pair[1])
        })
        .map(|(_, path)| shown(&path))
        .collect();

    assert!(
        found.is_empty(),
        "a kind folder holds its own kind: rules inside parsers is two places a reader has to \
         look for one thing. Found:\n{}",
        found.join("\n")
    );
}

#[test]
fn a_module_root_only_declares_and_re_exports() {
    let mut found = Vec::new();
    for (_, path) in every_path() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name != "mod.rs" && name != "lib.rs" {
            continue;
        }
        let text = fs::read_to_string(&path).expect("readable");
        for line in declaring_only(&text) {
            found.push(format!("{}:{line}", shown(&path)));
        }
    }

    assert!(
        found.is_empty(),
        "a mod.rs or a lib.rs declares modules and re-exports them, and the code is in the file \
         named after the thing. Found code at:\n{}",
        found.join("\n")
    );
}
