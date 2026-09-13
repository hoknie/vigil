#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Container,
    Socket,
}

const CONTAINER: &str = "container";

const SOCKET: &str = "container-socket";

impl Kind {
    pub fn of(key: &str) -> Option<Kind> {
        match key.split('|').next()? {
            CONTAINER => Some(Kind::Container),
            SOCKET => Some(Kind::Socket),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Container => CONTAINER,
            Kind::Socket => "socket",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_row_of_the_reading_is_named_by_the_word_its_key_opens_with() {
        assert_eq!(Kind::of("container|3ab1c0f2d4e5"), Some(Kind::Container));
        assert_eq!(
            Kind::of("container-socket|/run/docker.sock"),
            Some(Kind::Socket)
        );
    }

    #[test]
    fn a_row_of_another_collector_is_nothing_this_screen_draws() {
        assert_eq!(Kind::of("unix|/run/docker.sock"), None);
        assert_eq!(Kind::of("fs|/var"), None);
    }

    #[test]
    fn what_runs_is_read_before_the_socket_that_starts_it() {
        let mut order = vec![Kind::Socket, Kind::Container];
        order.sort();

        assert_eq!(order, vec![Kind::Container, Kind::Socket]);
    }
}
