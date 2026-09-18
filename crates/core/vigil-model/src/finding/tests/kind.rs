use crate::{Kind, KnownKind};

#[test]
fn a_closing_kind_names_the_kind_it_closes_and_the_pair_is_not_circular() {
    assert_eq!(
        KnownKind::PortListenRemoved.resolves(),
        Some(KnownKind::PortListenNew)
    );
    assert_eq!(KnownKind::PortListenNew.resolves(), None);

    for kind in KnownKind::ALL {
        if let Some(closed) = kind.resolves() {
            assert_eq!(
                closed.resolves(),
                None,
                "{} closes a closing kind",
                kind.as_str()
            );
            assert_ne!(closed, *kind, "{} closes itself", kind.as_str());
        }
    }
}

#[test]
fn the_cost_of_the_agent_going_over_its_ceiling_is_closed_by_its_own_kind() {
    assert_eq!(
        KnownKind::AgentBudgetRecovered.resolves(),
        Some(KnownKind::AgentBudgetExceeded),
        "coming back under the ceiling is an event of its own, not the absence of one"
    );
    assert_eq!(KnownKind::AgentBudgetExceeded.resolves(), None);
}

#[test]
fn a_firewall_that_came_back_closes_the_finding_that_it_was_gone() {
    assert_eq!(
        KnownKind::FirewallEnabled.resolves(),
        Some(KnownKind::FirewallDisabled),
        "a host that filters again is an event of its own, not the absence of one"
    );
    assert_eq!(KnownKind::FirewallDisabled.resolves(), None);
    assert_eq!(
        KnownKind::FirewallRulesetFlushed.resolves(),
        None,
        "rules that came back are not the rules that were there: nothing closes a flush"
    );
}

#[test]
fn a_collector_that_reads_again_closes_the_finding_that_it_could_not() {
    assert_eq!(
        KnownKind::AgentCollectorRecovered.resolves(),
        Some(KnownKind::AgentCollectorDegraded),
        "a collector that came back is an event of its own, and without it the finding \
         about the collector that went stands for the life of the host"
    );
    assert_eq!(KnownKind::AgentCollectorDegraded.resolves(), None);
}

#[test]
fn a_buffer_that_stopped_losing_findings_closes_the_finding_that_it_was_losing_them() {
    assert_eq!(
        KnownKind::AgentBufferDrained.resolves(),
        Some(KnownKind::AgentBufferDropping),
        "what is held is a number that only falls back to nothing; the fall is the event"
    );
    assert_eq!(KnownKind::AgentBufferDropping.resolves(), None);
}

#[test]
fn every_thing_the_agent_says_about_itself_that_can_end_has_a_kind_that_ends_it() {
    let opened_by_the_agent_and_ended_by_the_host = [
        KnownKind::AgentCollectorDegraded,
        KnownKind::AgentBufferDropping,
        KnownKind::AgentBudgetExceeded,
    ];

    for opening in opened_by_the_agent_and_ended_by_the_host {
        assert!(
            KnownKind::ALL
                .iter()
                .any(|kind| kind.resolves() == Some(opening)),
            "{} opens a finding nothing can close",
            opening.as_str()
        );
    }
}

#[test]
fn every_kind_round_trips_through_its_wire_form() {
    for kind in KnownKind::ALL {
        let wire = kind.as_str();
        assert_eq!(
            KnownKind::parse(wire),
            Some(*kind),
            "{wire} did not round-trip"
        );
    }
}

#[test]
fn wire_names_are_unique_and_namespaced() {
    let mut seen: Vec<&str> = KnownKind::ALL.iter().map(|k| k.as_str()).collect();
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), total, "two kinds share a wire name");
    for kind in KnownKind::ALL {
        assert!(
            kind.as_str().contains('.'),
            "{} is not <group>.<…>",
            kind.as_str()
        );
    }
}

#[test]
fn an_unknown_kind_survives_the_trip_instead_of_being_dropped() {
    let from_the_future = "port.listen.moved_to_wireguard";
    let parsed: Kind = from_the_future.to_string().into();
    assert_eq!(parsed, Kind::Unknown(from_the_future.to_string()));
    assert_eq!(String::from(parsed), from_the_future);
}

#[test]
fn the_one_thing_the_agent_does_to_the_host_has_a_kind_of_its_own_in_both_outcomes() {
    assert!(
        KnownKind::parse("agent.socket.killed").is_some()
            && KnownKind::parse("agent.socket.kill_refused").is_some(),
        "an agent that closed somebody's socket and reported nothing is an agent whose \
         journal disagrees with the host; the refusal is a finding too, because a kill \
         asked for and not carried out is the same question at an incident"
    );
    assert_eq!(
        KnownKind::AgentSocketKilled.resolves(),
        None,
        "a socket that came back is a new listening port, which has its own kind"
    );
}

#[test]
fn a_cron_job_that_went_away_closes_the_finding_that_it_had_appeared() {
    assert_eq!(
        KnownKind::PersistenceCronRemoved.resolves(),
        Some(KnownKind::PersistenceCronNew),
        "a line taken out of a crontab is the end of the job the agent reported, and a \
         finding about a job nobody can run any more stands open for the life of the host"
    );
    assert_eq!(KnownKind::PersistenceCronNew.resolves(), None);
    assert_eq!(
        KnownKind::PersistenceCronChanged.resolves(),
        None,
        "a job whose schedule changed is still the same job: nothing about it ended"
    );
}

#[test]
fn what_the_agent_does_to_what_starts_by_itself_has_a_kind_of_its_own_in_both_outcomes() {
    for named in [
        "agent.unit.controlled",
        "agent.unit.control_refused",
        "agent.cron.changed",
        "agent.cron.change_refused",
    ] {
        assert!(
            KnownKind::parse(named).is_some(),
            "{named} is missing: an agent that stopped somebody's service and reported \
             nothing is an agent whose journal disagrees with the host, and a refusal is \
             the same question at an incident as a stop that went through"
        );
    }
    assert_eq!(
        KnownKind::AgentUnitControlled.resolves(),
        None,
        "a unit started again is not the closing of the finding that it was stopped: both \
         are things a person did, and both stay in the journal"
    );
}

#[test]
fn the_agent_reports_on_itself_in_the_same_vocabulary() {
    assert!(
        KnownKind::ALL
            .iter()
            .any(|k| k.as_str().starts_with("agent.")),
        "a degraded collector must be expressible as a finding"
    );
}

#[test]
fn a_thing_a_container_engine_held_and_no_longer_holds_closes_the_finding_that_it_appeared() {
    for (removed, new) in [
        (
            KnownKind::ContainerImageRemoved,
            KnownKind::ContainerImageNew,
        ),
        (
            KnownKind::ContainerVolumeRemoved,
            KnownKind::ContainerVolumeNew,
        ),
        (
            KnownKind::ContainerNetworkRemoved,
            KnownKind::ContainerNetworkNew,
        ),
        (
            KnownKind::ContainerProjectRemoved,
            KnownKind::ContainerProjectNew,
        ),
        (KnownKind::ContainerPodRemoved, KnownKind::ContainerPodNew),
        (
            KnownKind::ContainerSecretRemoved,
            KnownKind::ContainerSecretNew,
        ),
    ] {
        assert_eq!(
            removed.resolves(),
            Some(new),
            "{} must close {}: an image deleted from the engine is the end of the image the \
             agent reported, and a finding about an image nobody can run any more stands open \
             for the life of the host",
            removed.as_str(),
            new.as_str()
        );
    }
    for changed in [
        KnownKind::ContainerImageChanged,
        KnownKind::ContainerVolumeChanged,
        KnownKind::ContainerNetworkChanged,
        KnownKind::ContainerProjectChanged,
        KnownKind::ContainerPodChanged,
        KnownKind::ContainerSecretChanged,
    ] {
        assert_eq!(
            changed.resolves(),
            None,
            "{}: a volume that changed its driver is still the same volume, and nothing about \
             it ended",
            changed.as_str()
        );
    }
}

#[test]
fn every_kind_the_engines_raise_is_named_by_family_thing_and_what_happened() {
    let engines: Vec<&str> = KnownKind::ALL
        .iter()
        .map(|kind| kind.as_str())
        .filter(|wire| wire.matches('.').count() == 2 && wire.starts_with("container."))
        .collect();

    assert_eq!(
        engines.len(),
        23,
        "{engines:?}: the vocabulary names what a container engine holds as \
         container.<thing>.<what happened>, the way persistence.cron.new does"
    );
    for wire in engines {
        let thing = wire.split('.').nth(1).unwrap_or_default();
        assert!(
            [
                "image", "volume", "network", "project", "pod", "secret", "registry"
            ]
            .contains(&thing),
            "{wire} names a thing no engine holds"
        );
    }
}
