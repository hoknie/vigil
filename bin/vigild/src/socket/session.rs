use std::io::{BufRead, ErrorKind, Read, Write};

use vigil_model::{ProtocolError, Request, Response, Rfc3339};

use super::{Shared, answer, change, control, kill};

const MAX_REQUEST_BYTES: u64 = 64 * 1024;

pub fn serve(
    reader: &mut dyn BufRead,
    writer: &mut dyn Write,
    shared: &Shared,
    now: &dyn Fn() -> Rfc3339,
) -> std::io::Result<()> {
    loop {
        let mut line = String::new();
        let read = match (&mut *reader).take(MAX_REQUEST_BYTES).read_line(&mut line) {
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::InvalidData => {
                writer.write_all(
                    Response::Error {
                        error: ProtocolError::new(
                            ProtocolError::MALFORMED_REQUEST,
                            "a request is UTF-8 text, one JSON document per line",
                        ),
                    }
                    .to_line()
                    .as_bytes(),
                )?;
                return writer.flush();
            }
            Err(error) => return Err(error),
        };

        if read == 0 {
            return Ok(());
        }

        if !line.ends_with('\n') {
            let refusal = Response::Error {
                error: ProtocolError::new(
                    ProtocolError::REQUEST_TOO_LONG,
                    format!("a request line is at most {MAX_REQUEST_BYTES} bytes"),
                ),
            };
            writer.write_all(refusal.to_line().as_bytes())?;
            return writer.flush();
        }

        let response = match Request::parse(line.trim_end()) {
            Ok(Request::Kill {
                sockets,
                programs,
                killing,
            }) => kill(&sockets, &programs, killing, shared, now()),
            Ok(Request::Change { changes }) => change(&changes, shared, now()),
            Ok(Request::Control { keys, controlling }) => {
                control(&keys, controlling, shared, now())
            }
            Ok(request) => shared.with(|state| answer(&request, state, now())),
            Err(error) => Response::Error { error },
        };

        writer.write_all(response.to_line().as_bytes())?;
        writer.flush()?;
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::super::fixture;
    use super::*;

    fn now() -> Rfc3339 {
        "2026-09-09T09:00:01.000Z".to_string()
    }

    fn talk(input: &str) -> Vec<Response> {
        let mut state = fixture::state();
        state.record_reading(fixture::reading(fixture::snapshot()));
        state.record_findings(&[fixture::finding("a new listening port")]);
        let shared = Shared::new(state);

        let mut written: Vec<u8> = Vec::new();
        serve(&mut Cursor::new(input), &mut written, &shared, &now)
            .expect("the buffers do not fail");

        String::from_utf8(written)
            .expect("utf-8")
            .lines()
            .map(|line| Response::parse(line).expect("one answer per line"))
            .collect()
    }

    #[test]
    fn one_question_per_line_gets_one_answer_per_line() {
        let answers = talk(
            "{\"query\":\"status\"}\n\
             {\"query\":\"snapshot\",\"collector\":\"network\"}\n\
             {\"query\":\"findings\",\"limit\":10}\n",
        );

        assert_eq!(answers.len(), 3);
        assert!(matches!(answers[0], Response::Status { .. }));
        match &answers[1] {
            Response::Snapshot { snapshot, .. } => {
                let snapshot = snapshot.as_ref().expect("there is a reading");
                assert!(snapshot.items.contains_key("tcp|0.0.0.0:4444"));
            }
            other => panic!("answered with {other:?}"),
        }
        match &answers[2] {
            Response::Findings { findings, .. } => assert_eq!(findings.len(), 1),
            other => panic!("answered with {other:?}"),
        }
    }

    #[test]
    fn a_line_that_is_not_a_request_is_answered_and_the_session_continues() {
        let answers = talk("nonsense\n{\"query\":\"status\"}\n");

        assert_eq!(answers.len(), 2);
        assert!(matches!(answers[0], Response::Error { .. }));
        assert!(matches!(answers[1], Response::Status { .. }));
    }

    #[test]
    fn a_line_longer_than_the_cap_is_refused_instead_of_buffered() {
        let flood = format!("{}\n", "x".repeat((MAX_REQUEST_BYTES + 1) as usize));

        let answers = talk(&flood);

        assert_eq!(answers.len(), 1);
        match &answers[0] {
            Response::Error { error } => assert_eq!(error.code, ProtocolError::REQUEST_TOO_LONG),
            other => panic!("answered with {other:?}"),
        }
    }

    #[test]
    fn nothing_a_client_can_send_changes_the_state_it_is_reading() {
        let mut state = fixture::state();
        state.record_findings(&[fixture::finding("a new listening port")]);
        let shared = Shared::new(state);
        let before = shared.with(|state| state.latest_findings(None).len());

        let mut written: Vec<u8> = Vec::new();
        serve(
            &mut Cursor::new(
                "{\"query\":\"status\"}\n{\"query\":\"findings\"}\n{\"query\":\"restart\"}\n",
            ),
            &mut written,
            &shared,
            &now,
        )
        .expect("the buffers do not fail");

        assert_eq!(
            shared.with(|state| state.latest_findings(None).len()),
            before
        );
    }
}
