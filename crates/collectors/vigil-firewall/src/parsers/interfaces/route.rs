pub const ROUTE: &str = "/proc/net/route";

const HEXADECIMAL: u32 = 16;

const FIELDS: usize = 8;

const DESTINATION_AT: usize = 1;

const GATEWAY_AT: usize = 2;

const MASK_AT: usize = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub name: String,
    pub network: u32,
    pub mask: u32,
    pub through: Option<u32>,
}

impl Route {
    pub fn is_the_way_out(&self) -> bool {
        self.mask == 0 && self.through.is_some()
    }

    pub fn holds(&self, address: u32) -> bool {
        self.mask != 0 && address & self.mask == self.network
    }
}

pub fn parse_route(text: &str) -> Vec<Route> {
    text.lines().skip(1).filter_map(route_of).collect()
}

pub fn written_out(address: u32) -> String {
    let octets = address.to_be_bytes();

    format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
}

fn route_of(line: &str) -> Option<Route> {
    let fields: Vec<&str> = line.split_ascii_whitespace().collect();
    if fields.len() < FIELDS {
        return None;
    }

    let gateway = little_endian(fields[GATEWAY_AT])?;

    Some(Route {
        name: fields[0].to_string(),
        network: little_endian(fields[DESTINATION_AT])?,
        mask: little_endian(fields[MASK_AT])?,
        through: match gateway {
            0 => None,
            through => Some(through),
        },
    })
}

fn little_endian(word: &str) -> Option<u32> {
    u32::from_str_radix(word, HEXADECIMAL)
        .ok()
        .map(u32::swap_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const A_HOST_WITH_ONE_WAY_OUT: &str = "\
Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT
eth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0
eth0\t0001A8C0\t00000000\t0001\t0\t0\t100\t00FFFFFF\t0\t0\t0
docker0\t000011AC\t00000000\t0001\t0\t0\t0\t0000FFFF\t0\t0\t0
";

    #[test]
    fn a_route_the_kernel_wrote_back_to_front_is_read_as_the_address_it_is() {
        let routes = parse_route(A_HOST_WITH_ONE_WAY_OUT);

        assert_eq!(routes.len(), 3);
        assert_eq!(written_out(routes[1].network), "192.168.1.0");
        assert_eq!(written_out(routes[1].mask), "255.255.255.0");
        assert_eq!(written_out(routes[2].network), "172.17.0.0");
    }

    #[test]
    fn the_route_with_no_network_of_its_own_is_the_one_a_packet_leaves_by() {
        let routes = parse_route(A_HOST_WITH_ONE_WAY_OUT);

        let out: Vec<&Route> = routes
            .iter()
            .filter(|route| route.is_the_way_out())
            .collect();

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "eth0");
        assert_eq!(
            out[0].through.map(written_out).as_deref(),
            Some("192.168.1.1"),
            "the graph draws where a packet goes when no other route claims it, and that is \
             this line and no other"
        );
    }

    #[test]
    fn an_address_is_placed_on_the_interface_whose_own_network_holds_it() {
        let routes = parse_route(A_HOST_WITH_ONE_WAY_OUT);
        let address = u32::from_be_bytes([192, 168, 1, 23]);

        assert!(routes[1].holds(address));
        assert!(!routes[2].holds(address));
        assert!(
            !routes[0].holds(address),
            "the way out holds every address there is, and an address placed on it would put \
             this host's own address on whichever interface the default route uses"
        );
    }

    #[test]
    fn a_line_the_kernel_wrote_in_a_shape_this_build_does_not_know_yields_no_route() {
        assert!(parse_route("").is_empty());
        assert!(parse_route("Iface Destination\neth0 zzzz 0 0 0 0 0 0 0 0 0\n").is_empty());
        assert!(parse_route("Iface Destination\neth0 00000000\n").is_empty());
    }
}
