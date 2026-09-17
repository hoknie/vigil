use crate::parsers::{Route, Traffic, written_out};

const LOOPBACK: &str = "lo";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Interface {
    pub name: String,
    pub addresses: Vec<String>,
    pub the_way_out: bool,
    pub traffic: Option<Traffic>,
}

impl Interface {
    pub fn gathered(
        counted: &[(String, Traffic)],
        routes: &[Route],
        local: &[u32],
        inet6: &[(String, String)],
    ) -> Vec<Interface> {
        let mut gathered: Vec<Interface> = counted
            .iter()
            .map(|(name, traffic)| Interface {
                name: name.clone(),
                addresses: addresses_of(name, routes, local, inet6),
                the_way_out: routes
                    .iter()
                    .any(|route| &route.name == name && route.is_the_way_out()),
                traffic: Some(*traffic),
            })
            .collect();

        gathered.sort_by(|left, right| {
            (left.is_the_loopback(), &left.name).cmp(&(right.is_the_loopback(), &right.name))
        });
        gathered
    }

    pub fn is_the_loopback(&self) -> bool {
        self.name == LOOPBACK
    }
}

fn addresses_of(
    name: &str,
    routes: &[Route],
    local: &[u32],
    inet6: &[(String, String)],
) -> Vec<String> {
    let mut found: Vec<String> = local
        .iter()
        .filter(|address| {
            routes
                .iter()
                .any(|route| route.name == name && route.holds(**address))
        })
        .map(|address| written_out(*address))
        .collect();

    found.extend(
        inet6
            .iter()
            .filter(|(held, _)| held == name)
            .map(|(_, address)| address.clone()),
    );
    found.dedup();
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::{parse_fib_trie, parse_net_dev, parse_route};

    const COUNTED: &str = "a\nb\n  \
        lo: 10 4 0 0 0 0 0 0 10 4 0 0 0 0 0 0\n  \
        eth0: 900 30 0 0 0 0 0 0 400 12 0 0 0 0 0 0\n";

    const ROUTED: &str = "Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT\n\
        eth0 00000000 0101A8C0 0003 0 0 100 00000000 0 0 0\n\
        eth0 0001A8C0 00000000 0001 0 0 100 00FFFFFF 0 0 0\n\
        lo 0000007F 00000000 0001 0 0 0 000000FF 0 0 0\n";

    const LOCAL: &str = "Local:\n  |-- 192.168.1.23\n     /32 host LOCAL\n  \
        |-- 127.0.0.1\n     /32 host LOCAL\n";

    fn gathered() -> Vec<Interface> {
        Interface::gathered(
            &parse_net_dev(COUNTED),
            &parse_route(ROUTED),
            &parse_fib_trie(LOCAL),
            &[("eth0".to_string(), "2001:db8::42/64".to_string())],
        )
    }

    #[test]
    fn an_interface_is_shown_with_the_address_this_host_answers_on_over_it() {
        let interfaces = gathered();

        assert_eq!(interfaces[0].name, "eth0");
        assert_eq!(
            interfaces[0].addresses,
            vec!["192.168.1.23", "2001:db8::42/64"]
        );
        assert_eq!(interfaces[1].name, "lo");
        assert_eq!(interfaces[1].addresses, vec!["127.0.0.1"]);
    }

    #[test]
    fn the_interface_a_packet_leaves_by_is_the_one_the_default_route_names() {
        let interfaces = gathered();

        assert!(interfaces[0].the_way_out);
        assert!(
            !interfaces[1].the_way_out,
            "a graph that draws every interface as the way out of this host draws no path at \
             all"
        );
    }

    #[test]
    fn the_loopback_is_listed_last_because_it_is_the_one_nothing_arrives_on() {
        let interfaces = gathered();

        assert_eq!(
            interfaces.last().map(|last| last.name.as_str()),
            Some("lo"),
            "a reader opens this graph to see what reaches this host from outside it"
        );
    }

    #[test]
    fn an_interface_whose_address_this_build_could_not_place_is_still_an_interface() {
        let nameless = Interface {
            name: "tun0".to_string(),
            ..Interface::default()
        };

        assert!(
            nameless.addresses.is_empty() && nameless.traffic.is_none(),
            "a tunnel this build cannot place an address on is an interface a packet still \
             arrives through, and leaving it out of the list hides the one nobody configured"
        );
    }
}
