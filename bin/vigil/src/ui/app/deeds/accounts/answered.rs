use vigil_model::{ProtocolError, Response};

use super::asked::Asked;

pub(super) fn answered(answer: Option<Response>) -> Asked {
    match answer {
        Some(Response::Changed { report }) => Asked::Report(*report),
        Some(Response::Error { error }) => Asked::Refused {
            advice: advice(&error.code),
            message: error.message,
        },
        Some(other) => Asked::Trouble(format!(
            "The agent answered a question nobody asked it: {other:?}"
        )),
        None => Asked::Trouble("The agent closed the connection without an answer.".to_string()),
    }
}

fn advice(code: &str) -> Option<String> {
    match code {
        ProtocolError::NOT_ALLOWED => Some(
            "Changing accounts from the console is off until vigil.yaml says \
             accounts.from_the_console: true, and the daemon reads that key once, at start-up."
                .to_string(),
        ),
        ProtocolError::UNKNOWN_QUERY => Some(
            "This agent is older than this console and changes no accounts: upgrade the agent."
                .to_string(),
        ),
        _ => None,
    }
}
