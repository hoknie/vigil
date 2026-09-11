use vigil_model::Finding;

#[derive(Default)]
pub struct Found {
    pub findings: Vec<Finding>,
    pub refused: Option<String>,
    pub dropped: u64,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_console_that_has_asked_nobody_holds_no_findings_and_claims_none_were_dropped() {
        let found = Found::default();

        assert!(found.findings.is_empty());
        assert_eq!(found.dropped, 0);
        assert!(found.refused.is_none());
    }
}
