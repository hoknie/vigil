use super::*;

#[test]
fn every_way_of_controlling_says_in_words_what_it_leaves_behind_on_the_host() {
    for controlling in Controlling::ALL {
        assert!(
            !controlling.said().is_empty(),
            "{} is offered to an operator with nothing written beside it, and a choice that \
             stops a service with no sentence on it is a choice made by accident",
            controlling.as_str()
        );
        assert_eq!(
            controlling.opposite().opposite(),
            *controlling,
            "{} is offered because a person can undo it from the same band; a way out that \
             does not come back is a way this console does not offer",
            controlling.as_str()
        );
    }
}

#[test]
fn stopping_a_unit_and_disabling_it_are_two_asks_and_not_one() {
    assert_eq!(Controlling::Stop.opposite(), Controlling::Start);
    assert_eq!(Controlling::Disable.opposite(), Controlling::Enable);
    assert_ne!(
        Controlling::Stop,
        Controlling::Disable,
        "systemctl stop leaves the unit enabled and the next boot starts it again; an agent \
         that treated them as one word would answer 'stopped' about a host that starts it \
         tomorrow"
    );
    assert!(Controlling::Stop.takes_something_away() && Controlling::Mask.takes_something_away());
    assert!(!Controlling::Start.takes_something_away());
}

#[test]
fn a_cron_job_is_never_offered_a_way_that_belongs_to_systemd() {
    assert_eq!(
        ControlTarget::Cron.ways(),
        &[Controlling::Comment, Controlling::Uncomment],
        "there is no systemctl for a line in a crontab, and a band whose every letter \
         answers with a refusal is a band that should not be drawn"
    );
    for way in ControlTarget::Cron.ways() {
        assert_eq!(way.target(), ControlTarget::Cron);
    }
    for way in ControlTarget::Unit.ways() {
        assert_eq!(way.target(), ControlTarget::Unit);
    }
}

#[test]
fn every_way_belongs_to_exactly_one_of_the_two_bands_it_can_be_asked_from() {
    for controlling in Controlling::ALL {
        let bands: Vec<&ControlTarget> = ControlTarget::ALL
            .iter()
            .filter(|target| target.ways().contains(controlling))
            .collect();

        assert_eq!(
            bands.len(),
            1,
            "{} is offered on {} band(s): a way nobody can reach is dead code, and a way on \
             two bands is a key whose meaning depends on where the cursor was",
            controlling.as_str(),
            bands.len()
        );
    }
}

#[test]
fn the_rows_a_band_can_act_on_are_told_apart_by_the_key_the_reading_gave_them() {
    assert!(ControlTarget::Unit.holds("unit|nginx.service"));
    assert!(ControlTarget::Unit.holds("timer|certbot.timer"));
    assert!(ControlTarget::Cron.holds("cron|/etc/crontab|root|/usr/bin/backup"));
    assert!(
        !ControlTarget::Unit.holds("cron|/etc/crontab|root|/usr/bin/backup"),
        "a cron line handed to systemctl is a unit name made of a whole command line"
    );
    assert!(
        !ControlTarget::Unit.holds("module|overlay")
            && !ControlTarget::Cron.holds("script|/etc/profile"),
        "the other rows of this reading are not units: nothing starts or stops them"
    );
}

#[test]
fn a_way_of_controlling_round_trips_through_the_wire_name_it_is_asked_by() {
    for controlling in Controlling::ALL {
        let line = serde_json::to_string(controlling).expect("serialises");
        assert_eq!(
            serde_json::from_str::<Controlling>(&line).expect("reads back"),
            *controlling
        );
    }
}

#[test]
fn a_refusal_carries_the_row_it_refused_and_the_reason_rather_than_being_left_out() {
    let report = ControlReport {
        target: ControlTarget::Unit,
        controlling: Controlling::Stop,
        acted_at: "2026-09-16T10:00:00.000Z".into(),
        controlled: vec![
            Controlled::done(
                "unit|nginx.service",
                Some("nginx.service".into()),
                "stopped",
            ),
            Controlled::refused("unit|dbus.service", "this unit is not in the last reading"),
        ],
    };

    assert_eq!(report.done(), 1);
    assert_eq!(report.refused(), 1);
    assert_eq!(
        report.controlled[1].key, "unit|dbus.service",
        "an operator who marked six rows and saw four stop has to be told which two are \
         still running, and why, by name"
    );
}

#[test]
fn a_report_from_an_agent_older_than_cron_is_read_as_a_report_about_units() {
    let report: ControlReport = serde_json::from_str(
        "{\"controlling\":\"stop\",\"acted_at\":\"2026-09-16T10:00:00.000Z\",\"controlled\":[]}",
    )
    .expect("reads");

    assert_eq!(report.target, ControlTarget::Unit);
}
