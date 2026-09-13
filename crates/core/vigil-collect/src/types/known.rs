#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownCollector {
    pub name: &'static str,
    pub subject: &'static str,
    pub every_seconds: u32,
    pub unit: Option<&'static str>,
}
