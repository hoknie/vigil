use vigil_model::Severity;

use super::SyslogFacility;

pub fn of(severity: &Severity) -> u8 {
    match severity {
        Severity::Critical => 2,
        Severity::High => 3,
        Severity::Medium => 4,
        Severity::Low => 5,
        Severity::Info => 6,
        Severity::Unknown(_) => 4,
    }
}

pub fn priority(facility: SyslogFacility, severity: &Severity) -> u8 {
    facility.code() * 8 + of(severity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_levels_this_agent_can_produce_all_land_between_crit_and_info() {
        for severity in Severity::KNOWN {
            let urgency = of(severity);
            assert!(
                (2..=6).contains(&urgency),
                "{severity} became {urgency}: emerg and alert belong to the host, not to us"
            );
        }
    }

    #[test]
    fn a_worse_finding_is_never_a_quieter_line() {
        let order = [
            Severity::Info,
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ];
        for pair in order.windows(2) {
            assert!(
                of(&pair[1]) < of(&pair[0]),
                "{} must be louder than {}",
                pair[1],
                pair[0]
            );
        }
    }

    #[test]
    fn a_severity_this_build_does_not_know_stays_above_the_level_people_filter_out() {
        let unknown = Severity::Unknown("catastrophic".into());

        assert_eq!(of(&unknown), 4);
        assert!(
            of(&unknown) < of(&Severity::Info),
            "an unrecognised level must not be dropped by a collector that samples info"
        );
    }

    #[test]
    fn the_priority_carries_the_facility_and_the_urgency_together() {
        let local4 = SyslogFacility::parse("local4").expect("known");

        assert_eq!(priority(local4, &Severity::Critical), 20 * 8 + 2);
        assert_eq!(priority(local4, &Severity::Info), 20 * 8 + 6);
    }
}
