use std::collections::BTreeMap;

use vigil_model::Snapshot;

use crate::parsers::{
    MacosFirewallReading, Pf, Traffic, Understood, macos_firewall_snapshot, understood,
};
use crate::types::{
    ANCHOR_NAT, ANCHOR_RULES, ANSWERED, APPLICATION_FIREWALL, Answer, FAILED, FirewallDump,
    Interface, PF_INFO,
};

pub const PF_INFO_ENABLED: &str = "\
Status: Enabled for 0 days 02:14:51           Debug: Urgent

State Table                          Total             Rate
  current entries                        7               
  searches                          118273           14.7/s
  inserts                             2310            0.3/s
  removals                            2303            0.3/s
Counters
  match                               4520            0.6/s
  bad-offset                             0            0.0/s
";

pub const PF_INFO_DISABLED: &str = "\
Status: Disabled for 0 days 00:00:12          Debug: Urgent

State Table                          Total             Rate
  current entries                        0               
";

pub const PF_RULES_PRINTED: &str = "\
scrub-anchor \"com.apple/*\" all fragment reassemble
anchor \"com.apple/*\" all
block drop in all
pass out all flags S/SA keep state
pass in quick on lo0 all flags S/SA keep state
pass in proto tcp from any to any port = 22 flags S/SA keep state
block drop in quick on en0 proto tcp from <bruteforce> to any
";

pub const PF_NAT_PRINTED: &str = "\
nat-anchor \"com.apple/*\" all
rdr-anchor \"com.apple/*\" all
nat on en0 inet from 192.168.64.0/24 to any -> (en0) round-robin
rdr pass on en0 inet proto tcp from any to any port = 8080 -> 127.0.0.1 port 80
";

pub const PF_ANCHORS_PRINTED: &str = "  com.apple
  com.apple/200.AirDrop
  com.apple/200.AirDrop/Bonjour
  com.apple/250.ApplicationFirewall
";

pub const APPLE_ANCHOR: &str = "\
anchor \"200.AirDrop/*\" all
anchor \"250.ApplicationFirewall/*\" all
";

pub const BONJOUR_ANCHOR: &str = "\
pass in quick on awdl0 inet6 proto udp from any to any port = 5353 keep state
pass out quick on awdl0 inet6 proto udp from any to any port = 5353 keep state
";

pub const SOCKETFILTERFW: &str = "\
Firewall is enabled. (State = 1)
Firewall has block all state set to disabled.
Firewall stealth mode is on
Automatically allow built-in signed software ENABLED.
Automatically allow downloaded signed software DISABLED.
Total number of apps = 3 

1 : /usr/sbin/cupsd 
             (Allow incoming connections)

2 : /Applications/Google Chrome.app 
             (Allow incoming connections)

3 : /usr/local/bin/node 
             (Block incoming connections)
";

pub const PFCTL_DENIED: &str = "pfctl: /dev/pf: Permission denied";

fn answered(program: &str, arguments: &[&str], printed: &str) -> Answer {
    Answer {
        state: ANSWERED.to_string(),
        program: program.to_string(),
        arguments: arguments.iter().map(|word| (*word).to_string()).collect(),
        milliseconds: 4,
        printed: printed.to_string(),
        truncated: false,
        status: Some(0),
        why: None,
    }
}

fn refused(arguments: &[&str]) -> Answer {
    Answer {
        state: FAILED.to_string(),
        program: "/sbin/pfctl".to_string(),
        arguments: arguments.iter().map(|word| (*word).to_string()).collect(),
        milliseconds: 2,
        printed: String::new(),
        truncated: false,
        status: Some(1),
        why: Some(PFCTL_DENIED.to_string()),
    }
}

pub fn pf_dump(enabled: bool) -> FirewallDump {
    let pfctl = "/sbin/pfctl";
    let mut asked = BTreeMap::new();
    let info = match enabled {
        true => PF_INFO_ENABLED,
        false => PF_INFO_DISABLED,
    };
    asked.insert(PF_INFO.to_string(), answered(pfctl, &["-s", "info"], info));
    asked.insert(
        crate::types::PF_RULES.to_string(),
        answered(pfctl, &["-s", "rules"], PF_RULES_PRINTED),
    );
    asked.insert(
        crate::types::PF_NAT.to_string(),
        answered(pfctl, &["-s", "nat"], PF_NAT_PRINTED),
    );
    asked.insert(
        crate::types::PF_ANCHORS.to_string(),
        answered(pfctl, &["-v", "-s", "Anchors"], PF_ANCHORS_PRINTED),
    );
    for (anchor, rules) in [
        ("com.apple", APPLE_ANCHOR),
        ("com.apple/200.AirDrop", ""),
        ("com.apple/200.AirDrop/Bonjour", BONJOUR_ANCHOR),
        ("com.apple/250.ApplicationFirewall", ""),
    ] {
        asked.insert(
            format!("{ANCHOR_RULES}{anchor}"),
            answered(pfctl, &["-a", anchor, "-s", "rules"], rules),
        );
        asked.insert(
            format!("{ANCHOR_NAT}{anchor}"),
            answered(pfctl, &["-a", anchor, "-s", "nat"], ""),
        );
    }
    asked.insert(
        APPLICATION_FIREWALL.to_string(),
        answered(
            "/usr/libexec/ApplicationFirewall/socketfilterfw",
            &["--getglobalstate", "--listapps"],
            SOCKETFILTERFW,
        ),
    );

    FirewallDump {
        taken_at: "2026-09-19T09:00:00.000Z".to_string(),
        written_by: "/usr/local/libexec/vigil/vigil-firewall-dump".to_string(),
        deadline_seconds: 10,
        asked,
    }
}

pub fn unprivileged_dump() -> FirewallDump {
    let mut dump = pf_dump(true);
    for (key, answer) in dump.asked.iter_mut() {
        if key != APPLICATION_FIREWALL {
            *answer = refused(&[]);
        }
    }
    dump
}

fn interfaces() -> Vec<Interface> {
    vec![
        Interface {
            name: "en0".to_string(),
            addresses: vec![
                "192.168.1.23".to_string(),
                "fe80::1c2b:3a4d:5e6f:7081/64".to_string(),
            ],
            the_way_out: true,
            traffic: Some(Traffic {
                packets_in: 318_841,
                bytes_in: 91_200_311,
                dropped_in: 7,
                packets_out: 201_773,
                bytes_out: 12_084_411,
                dropped_out: 0,
            }),
        },
        Interface {
            name: "utun0".to_string(),
            addresses: Vec::new(),
            the_way_out: false,
            traffic: Some(Traffic::default()),
        },
        Interface {
            name: "lo0".to_string(),
            addresses: vec!["127.0.0.1".to_string(), "::1/128".to_string()],
            the_way_out: false,
            traffic: Some(Traffic {
                packets_in: 41_220,
                bytes_in: 5_140_552,
                dropped_in: 0,
                packets_out: 41_220,
                bytes_out: 5_140_552,
                dropped_out: 0,
            }),
        },
    ]
}

pub fn firewall_on_macos() -> Snapshot {
    let Understood {
        pf,
        application_firewall,
    } = understood(&pf_dump(true));
    let pf = pf.expect("the sample pf reads");
    let application_firewall = application_firewall.expect("the sample socketfilterfw reads");

    macos_firewall_snapshot(
        "2026-09-19T09:00:00.000Z",
        &MacosFirewallReading {
            pf: Some(Pf {
                enabled: pf.enabled,
                main: &pf.main,
                anchors: &pf.anchors,
            }),
            application_firewall: Some(&application_firewall),
            interfaces: &interfaces(),
            counting: true,
        },
    )
}
