const STATUS: &str = "Status:";

pub fn parse_pf_info(printed: &str) -> Option<bool> {
    let status = printed
        .lines()
        .map(str::trim_start)
        .find_map(|line| line.strip_prefix(STATUS))?;

    match status.split_whitespace().next()? {
        "Enabled" => Some(true),
        "Disabled" => Some(false),
        _ => None,
    }
}
