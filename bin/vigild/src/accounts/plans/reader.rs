pub type Reader<'a> = &'a dyn Fn(&str, Option<u32>) -> Result<Option<String>, String>;

pub fn present(path: &str, holder: Option<u32>, read: Reader<'_>) -> Result<String, String> {
    read(path, holder)?
        .ok_or_else(|| format!("{path} is no longer there: the reading is older than it"))
}
