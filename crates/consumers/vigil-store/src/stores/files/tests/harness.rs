use std::fs;
use std::path::PathBuf;

pub(super) fn temporary_directory(name: &str) -> PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-store-{}-{name}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos()
                + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&directory);
    directory
}
