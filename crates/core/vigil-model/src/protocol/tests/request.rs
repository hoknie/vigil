use crate::{AccountChange, Controlling, KillTarget, Killing, ProtocolError, Request};

#[test]
fn three_requests_do_something_on_the_host_and_all_three_are_named_here() {
    assert_eq!(Request::READING, &["status", "snapshot", "findings"]);
    for name in Request::READING {
        assert!(
            Request::parse(&format!(
                "{{\"query\":\"{name}\",\"collector\":\"network\"}}"
            ))
            .is_ok(),
            "{name} is listed but does not parse"
        );
        let request = Request::parse(&format!(
            "{{\"query\":\"{name}\",\"collector\":\"network\"}}"
        ))
        .expect("parses");
        assert!(
            !request.acts_on_the_host(),
            "{name} answers a question and must never grow a side effect"
        );
    }

    let acting: Vec<&&str> = Request::NAMES
        .iter()
        .filter(|name| !Request::READING.contains(name))
        .collect();

    assert_eq!(
        acting,
        vec![&"kill", &"change", &"control"],
        "this protocol held three questions and no verb until 2026-09-14, when the owner \
         put in kill on purpose, and later that day change, for the accounts of this host, \
         with the decision taken again; control, for what this host starts by itself, was \
         taken the same way on 2026-09-16 \
         (docs/designs/2026-09-16-DESIGN-console-units.md). A fourth verb arriving without \
         that decision being taken a fourth time is the failure this test exists to make \
         loud."
    );
}

#[test]
fn a_change_names_the_object_and_what_to_do_to_it_and_no_command_line() {
    let request = Request::parse(
        "{\"query\":\"change\",\"changes\":[{\"change\":\"delete_user\",\"name\":\"eve\"}]}",
    )
    .expect("parses");

    assert!(request.acts_on_the_host());
    assert_eq!(
        request,
        Request::Change {
            changes: vec![AccountChange::DeleteUser { name: "eve".into() }]
        }
    );
    assert!(
        Request::parse(
            "{\"query\":\"change\",\"changes\":[{\"change\":\"run\",\"argv\":[\"/bin/sh\"]}]}"
        )
        .is_err(),
        "the daemon is asked for one of the changes it knows by name; there is no shape \
         of this request that carries a program to start"
    );
}

#[test]
fn the_one_verb_names_what_it_kills_and_how_and_carries_nothing_a_daemon_would_run() {
    let request = Request::parse(
        "{\"query\":\"kill\",\"sockets\":[\"tcp|0.0.0.0:4444\"],\"killing\":\"terminate\"}",
    )
    .expect("parses");

    match &request {
        Request::Kill {
            sockets,
            programs,
            killing,
        } => {
            assert_eq!(sockets, &vec!["tcp|0.0.0.0:4444".to_string()]);
            assert!(programs.is_empty());
            assert_eq!(*killing, Killing::Terminate);
        }
        other => panic!("parsed as {other:?}"),
    }
    assert!(request.acts_on_the_host());
}

#[test]
fn a_program_is_stopped_by_the_same_verb_and_not_by_a_second_one() {
    let request = Request::kill(
        KillTarget::Program,
        vec!["exec|/tmp/.x/nc|www-data".into()],
        Killing::Kill,
    );
    let line = request.to_line();

    assert!(line.contains("\"query\":\"kill\""), "{line}");
    assert!(line.contains("\"programs\""), "{line}");
    assert_eq!(Request::parse(line.trim_end()).expect("parses"), request);
}

#[test]
fn a_kill_written_before_programs_could_be_named_still_reads_as_a_kill_of_sockets() {
    let request = Request::parse(
        "{\"query\":\"kill\",\"sockets\":[\"tcp|0.0.0.0:4444\"],\"killing\":\"kill\"}",
    )
    .expect("parses");

    assert_eq!(
        request,
        Request::kill(
            KillTarget::Socket,
            vec!["tcp|0.0.0.0:4444".into()],
            Killing::Kill
        )
    );
}

#[test]
fn a_way_of_killing_this_build_has_not_heard_of_is_refused_rather_than_guessed_at() {
    let error = Request::parse(
        "{\"query\":\"kill\",\"sockets\":[\"tcp|0.0.0.0:80\"],\"killing\":\"reboot\"}",
    )
    .expect_err("must not be accepted");

    assert_eq!(error.code, ProtocolError::MALFORMED_REQUEST);
}

#[test]
fn a_control_names_rows_of_a_reading_and_one_word_this_build_knows_and_no_command_line() {
    let request = Request::parse(
        "{\"query\":\"control\",\"keys\":[\"unit|nginx.service\"],\"controlling\":\"stop\"}",
    )
    .expect("parses");

    assert!(request.acts_on_the_host());
    assert_eq!(
        request,
        Request::control(vec!["unit|nginx.service".into()], Controlling::Stop)
    );
    assert!(
        Request::parse(
            "{\"query\":\"control\",\"keys\":[\"nginx\"],\"controlling\":\"restart --now\"}"
        )
        .is_err(),
        "the daemon is asked for one of the words it knows by name; there is no shape of \
         this request that carries an argument for systemctl, let alone a command line"
    );
}

#[test]
fn a_way_of_controlling_this_build_has_not_heard_of_is_refused_rather_than_guessed_at() {
    let error = Request::parse(
        "{\"query\":\"control\",\"keys\":[\"unit|nginx.service\"],\"controlling\":\"reboot\"}",
    )
    .expect_err("must not be accepted");

    assert_eq!(error.code, ProtocolError::MALFORMED_REQUEST);
}

#[test]
fn a_request_round_trips_through_its_line() {
    for request in [
        Request::Status,
        Request::Snapshot {
            collector: "network".into(),
        },
        Request::Findings { limit: Some(20) },
        Request::Findings { limit: None },
        Request::kill(
            KillTarget::Socket,
            vec!["tcp|0.0.0.0:4444".into()],
            Killing::Destroy,
        ),
        Request::kill(
            KillTarget::Program,
            vec!["exec|/usr/sbin/nginx|root".into()],
            Killing::Terminate,
        ),
        Request::control(vec!["timer|certbot.timer".into()], Controlling::Disable),
        Request::control(
            vec!["cron|/etc/crontab|root|/usr/bin/backup".into()],
            Controlling::Comment,
        ),
    ] {
        let line = request.to_line();
        assert!(line.ends_with('\n'), "{line:?} is not one line");
        assert_eq!(Request::parse(line.trim_end()).expect("parses"), request);
    }
}

#[test]
fn an_unknown_query_is_refused_by_name_instead_of_ignored() {
    let error = Request::parse("{\"query\":\"restart\"}").expect_err("must not be accepted");

    assert_eq!(error.code, ProtocolError::UNKNOWN_QUERY);
    assert!(error.message.contains("restart"), "{error}");
    assert!(error.message.contains("status"), "{error}");
}

#[test]
fn text_that_is_not_a_request_is_answered_rather_than_guessed_at() {
    for line in ["", "not json", "[1,2,3]", "\"status\"", "{}"] {
        let error = Request::parse(line).expect_err("must not be accepted: {line:?}");
        assert!(
            error.code == ProtocolError::MALFORMED_REQUEST
                || error.code == ProtocolError::UNKNOWN_QUERY,
            "{line:?} produced {error}"
        );
    }
}

#[test]
fn a_query_missing_what_it_needs_says_which_field() {
    let error = Request::parse("{\"query\":\"snapshot\"}").expect_err("collector is needed");

    assert_eq!(error.code, ProtocolError::MALFORMED_REQUEST);
    assert!(error.message.contains("collector"), "{error}");
}

#[test]
fn a_field_from_a_newer_console_does_not_make_the_request_unreadable() {
    let parsed = Request::parse("{\"query\":\"findings\",\"limit\":5,\"since\":\"tomorrow\"}")
        .expect("parses");

    assert_eq!(parsed, Request::Findings { limit: Some(5) });
}
