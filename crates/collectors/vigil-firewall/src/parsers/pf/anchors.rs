use crate::types::{MOST_ANCHORS, is_an_anchor};

pub fn parse_pf_anchors(printed: &str) -> Vec<String> {
    let mut anchors: Vec<String> = printed
        .lines()
        .map(str::trim)
        .filter(|line| is_an_anchor(line))
        .map(str::to_string)
        .collect();
    anchors.sort();
    anchors.dedup();
    anchors.truncate(MOST_ANCHORS);
    anchors
}
