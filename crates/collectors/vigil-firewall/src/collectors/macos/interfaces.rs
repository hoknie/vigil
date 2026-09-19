use std::collections::BTreeMap;
use std::ffi::CStr;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::{self, NonNull};

use vigil_collect::sysctl_numbered;

use crate::parsers::{Traffic, parse_default_routes};
use crate::types::Interface;

const LOOPBACK: &str = "lo0";

pub(super) fn interfaces() -> Vec<Interface> {
    let mut found: BTreeMap<String, Interface> = BTreeMap::new();
    let mut listed: *mut libc::ifaddrs = ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut listed) } != 0 {
        return Vec::new();
    }

    let mut at = NonNull::new(listed);
    while let Some(node) = at {
        let entry = unsafe { node.as_ref() };
        at = NonNull::new(entry.ifa_next);
        if entry.ifa_name.is_null() || entry.ifa_addr.is_null() {
            continue;
        }
        let name = unsafe { CStr::from_ptr(entry.ifa_name) }
            .to_string_lossy()
            .into_owned();
        let interface = found.entry(name.clone()).or_insert_with(|| Interface {
            name,
            ..Interface::default()
        });

        match i32::from(unsafe { ptr::read_unaligned(entry.ifa_addr) }.sa_family) {
            libc::AF_LINK if !entry.ifa_data.is_null() => {
                let data = unsafe { ptr::read_unaligned(entry.ifa_data as *const libc::if_data) };
                interface.traffic = Some(Traffic {
                    packets_in: u64::from(data.ifi_ipackets),
                    bytes_in: u64::from(data.ifi_ibytes),
                    dropped_in: u64::from(data.ifi_iqdrops),
                    packets_out: u64::from(data.ifi_opackets),
                    bytes_out: u64::from(data.ifi_obytes),
                    dropped_out: 0,
                });
            }
            libc::AF_INET => {
                let address =
                    unsafe { ptr::read_unaligned(entry.ifa_addr as *const libc::sockaddr_in) };
                interface
                    .addresses
                    .push(Ipv4Addr::from(u32::from_be(address.sin_addr.s_addr)).to_string());
            }
            libc::AF_INET6 => {
                let address =
                    unsafe { ptr::read_unaligned(entry.ifa_addr as *const libc::sockaddr_in6) };
                let prefix = match entry.ifa_netmask.is_null() {
                    true => 128,
                    false => {
                        let mask = unsafe {
                            ptr::read_unaligned(entry.ifa_netmask as *const libc::sockaddr_in6)
                        };
                        mask.sin6_addr
                            .s6_addr
                            .iter()
                            .map(|byte| byte.count_ones())
                            .sum::<u32>()
                    }
                };
                interface.addresses.push(format!(
                    "{}/{prefix}",
                    Ipv6Addr::from(address.sin6_addr.s6_addr)
                ));
            }
            _ => {}
        }
    }
    unsafe { libc::freeifaddrs(listed) };

    for index in default_routes() {
        let mut name = [0 as libc::c_char; libc::IF_NAMESIZE];
        if unsafe { libc::if_indextoname(u32::from(index), name.as_mut_ptr()) }.is_null() {
            continue;
        }
        let name = unsafe { CStr::from_ptr(name.as_ptr()) }.to_string_lossy();
        if let Some(interface) = found.get_mut(name.as_ref()) {
            interface.the_way_out = true;
        }
    }

    let mut gathered: Vec<Interface> = found.into_values().collect();
    gathered.sort_by(|left, right| {
        (left.name == LOOPBACK, &left.name).cmp(&(right.name == LOOPBACK, &right.name))
    });
    gathered
}

fn default_routes() -> Vec<u16> {
    let mut indices = Vec::new();
    for family in [libc::AF_INET, libc::AF_INET6] {
        let asked = [
            libc::CTL_NET,
            libc::PF_ROUTE,
            0,
            family,
            libc::NET_RT_FLAGS,
            libc::RTF_GATEWAY,
        ];
        if let Ok(dumped) = sysctl_numbered(&asked) {
            for index in parse_default_routes(&dumped) {
                if !indices.contains(&index) {
                    indices.push(index);
                }
            }
        }
    }
    indices
}
