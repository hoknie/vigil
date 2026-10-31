use std::net::IpAddr;

pub const CEILING: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    pub address: IpAddr,
    pub up: bool,
    pub loopback: bool,
}

pub fn addresses() -> Vec<String> {
    chosen(&seen())
}

pub fn chosen(seen: &[Seen]) -> Vec<String> {
    let mut kept: Vec<IpAddr> = seen
        .iter()
        .filter(|one| one.up && !one.loopback && of_the_host(one.address))
        .map(|one| one.address)
        .collect();
    kept.sort();
    kept.dedup();
    kept.truncate(CEILING);
    kept.into_iter()
        .map(|address| address.to_string())
        .collect()
}

fn of_the_host(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(v4) => !v4.is_loopback() && !v4.is_link_local() && !v4.is_unspecified(),
        IpAddr::V6(v6) => !v6.is_loopback() && !v6.is_unicast_link_local() && !v6.is_unspecified(),
    }
}

#[cfg(unix)]
fn seen() -> Vec<Seen> {
    use std::net::{Ipv4Addr, Ipv6Addr};

    let mut first: *mut libc::ifaddrs = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut first) } != 0 {
        return Vec::new();
    }

    let mut seen = Vec::new();
    let mut at = first;
    while !at.is_null() {
        let entry = unsafe { &*at };
        at = entry.ifa_next;
        if entry.ifa_addr.is_null() {
            continue;
        }
        let flags = entry.ifa_flags as u64;
        let up = flags & libc::IFF_UP as u64 != 0;
        let loopback = flags & libc::IFF_LOOPBACK as u64 != 0;

        let family = i32::from(unsafe { (*entry.ifa_addr).sa_family });
        let address = match family {
            libc::AF_INET => {
                let socket = unsafe { &*(entry.ifa_addr as *const libc::sockaddr_in) };
                IpAddr::V4(Ipv4Addr::from(u32::from_be(socket.sin_addr.s_addr)))
            }
            libc::AF_INET6 => {
                let socket = unsafe { &*(entry.ifa_addr as *const libc::sockaddr_in6) };
                IpAddr::V6(Ipv6Addr::from(socket.sin6_addr.s6_addr))
            }
            _ => continue,
        };
        seen.push(Seen {
            address,
            up,
            loopback,
        });
    }

    unsafe { libc::freeifaddrs(first) };
    seen
}

#[cfg(not(unix))]
fn seen() -> Vec<Seen> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn up(address: &str) -> Seen {
        Seen {
            address: address.parse().expect("an address"),
            up: true,
            loopback: false,
        }
    }

    #[test]
    fn a_loopback_address_is_not_an_address_of_the_host() {
        let mut lo = up("127.0.0.1");
        lo.loopback = true;

        assert_eq!(
            chosen(&[lo, up("::1"), up("127.0.1.1"), up("10.0.4.17")]),
            ["10.0.4.17"]
        );
    }

    #[test]
    fn a_link_local_address_is_not_an_address_of_the_host() {
        assert_eq!(
            chosen(&[
                up("fe80::1c2a:5ff:fe00:1"),
                up("169.254.10.1"),
                up("2001:db8::5")
            ]),
            ["2001:db8::5"]
        );
    }

    #[test]
    fn an_address_on_an_interface_that_is_down_is_not_sent() {
        let mut down = up("192.168.7.2");
        down.up = false;

        assert!(chosen(&[down]).is_empty());
    }

    #[test]
    fn addresses_are_sorted_without_repeats_and_cut_at_thirty_two() {
        let mut many: Vec<Seen> = (1..=40).rev().map(|n| up(&format!("10.0.0.{n}"))).collect();
        many.push(up("10.0.0.3"));

        let kept = chosen(&many);

        assert_eq!(kept.len(), CEILING);
        assert_eq!(kept.first().map(String::as_str), Some("10.0.0.1"));
        assert_eq!(
            kept.iter().filter(|address| *address == "10.0.0.3").count(),
            1
        );
    }

    #[test]
    fn the_addresses_of_this_host_are_read_and_loopback_is_not_among_them() {
        let read = addresses();

        assert!(
            !read
                .iter()
                .any(|address| address == "127.0.0.1" || address == "::1")
        );
        assert!(read.len() <= CEILING);
    }

    #[test]
    fn reading_the_addresses_of_this_host_costs_microseconds() {
        const CALLS: u32 = 1_000;
        let started = std::time::Instant::now();
        for _ in 0..CALLS {
            std::hint::black_box(addresses());
        }
        let each = started.elapsed() / CALLS;

        println!(
            "addresses per call over {CALLS}: {each:?}, {} address(es)",
            addresses().len()
        );
        assert!(
            each < std::time::Duration::from_millis(1),
            "read once per delivery, a call costing {each:?} is no longer a rounding error"
        );
    }
}
