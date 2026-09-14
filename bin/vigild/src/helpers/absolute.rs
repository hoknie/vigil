pub fn of(path: &str) -> String {
    std::fs::canonicalize(path)
        .map(|whole| whole.display().to_string())
        .unwrap_or_else(|_| path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_named_from_a_working_directory_is_answered_with_as_a_whole_path() {
        let directory = std::env::temp_dir().join(format!("vigil-absolute-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("temp dir");
        let path = directory.join("vigil.yaml");
        std::fs::write(&path, "{}\n").expect("writes");

        let answered = of(path.to_str().expect("utf-8"));

        assert!(answered.starts_with('/'), "{answered}");
        assert!(answered.ends_with("vigil.yaml"), "{answered}");
    }

    #[test]
    fn a_path_that_cannot_be_resolved_is_answered_with_as_it_was_written() {
        assert_eq!(
            of("config/vigil.example.yaml"),
            "config/vigil.example.yaml",
            "a console reading this has to be told what the daemon was given, not a guess"
        );
    }
}
