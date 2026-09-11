use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use vigil_model::{Request, Response};

use super::{Trouble, TroubleKind};

const TIMEOUT: Duration = Duration::from_secs(3);

pub struct Link {
    path: String,
}

impl Link {
    pub fn new(path: impl Into<String>) -> Self {
        Link { path: path.into() }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn ask(&self, requests: &[Request]) -> Result<Vec<Response>, Trouble> {
        let stream = UnixStream::connect(&self.path).map_err(|error| {
            let kind = match error.kind() {
                ErrorKind::PermissionDenied => TroubleKind::Forbidden,
                _ => TroubleKind::Absent,
            };
            Trouble::new(&self.path, kind, error.to_string())
        })?;

        let _ = stream.set_read_timeout(Some(TIMEOUT));
        let _ = stream.set_write_timeout(Some(TIMEOUT));

        let mut writing = &stream;
        for request in requests {
            writing
                .write_all(request.to_line().as_bytes())
                .map_err(|error| self.trouble(TroubleKind::Absent, error.to_string()))?;
        }
        writing
            .flush()
            .map_err(|error| self.trouble(TroubleKind::Absent, error.to_string()))?;

        let mut reader = BufReader::new(&stream);
        let mut answers = Vec::with_capacity(requests.len());
        for _ in requests {
            let mut line = String::new();
            let read = reader
                .read_line(&mut line)
                .map_err(|error| self.trouble(TroubleKind::Absent, error.to_string()))?;
            if read == 0 {
                return Err(self.trouble(
                    TroubleKind::Absent,
                    "the connection closed before every question was answered",
                ));
            }
            answers.push(
                Response::parse(line.trim_end())
                    .map_err(|error| self.trouble(TroubleKind::Unreadable, error.to_string()))?,
            );
        }

        Ok(answers)
    }

    fn trouble(&self, kind: TroubleKind, cause: impl Into<String>) -> Trouble {
        Trouble::new(&self.path, kind, cause)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_socket_that_is_not_there_is_a_sentence_rather_than_a_panic() {
        let link = Link::new("/nonexistent/vigil/does-not-exist.sock");

        let trouble = link
            .ask(&[Request::Status])
            .expect_err("nothing is listening");

        assert_eq!(trouble.kind, TroubleKind::Absent);
        assert!(
            trouble.headline().contains("does-not-exist.sock"),
            "{trouble}"
        );
        assert!(
            !trouble.what_to_try().is_empty(),
            "a dead end must come with the next thing to try"
        );
    }
}
