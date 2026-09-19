use serde_json::from_slice;

use super::application_firewall::{ApplicationFirewall, parse_application_firewall};
use super::pf::{MAIN, PfRuleset, parse_pf_anchors, parse_pf_info, parse_pf_rules};
use crate::types::{
    ANCHOR_NAT, ANCHOR_RULES, APPLICATION_FIREWALL, FirewallDump, PF_ANCHORS, PF_INFO, PF_NAT,
    PF_RULES,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PfRead {
    pub enabled: bool,
    pub main: PfRuleset,
    pub anchors: Vec<PfRuleset>,
    pub anchors_unread: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Understood {
    pub pf: Result<PfRead, String>,
    pub application_firewall: Result<ApplicationFirewall, String>,
}

pub fn parse_firewall_dump(bytes: &[u8]) -> Result<FirewallDump, String> {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(
            "the file is empty: the job was started and wrote nothing, so what this Mac filters \
             is unknown"
                .to_string(),
        );
    }
    from_slice(bytes).map_err(|error| {
        format!("the file is not the document vigil-firewall-dump writes ({error})")
    })
}

pub fn understood(dump: &FirewallDump) -> Understood {
    Understood {
        pf: pf_of(dump),
        application_firewall: dump.printed(APPLICATION_FIREWALL).and_then(|printed| {
            parse_application_firewall(printed).ok_or_else(|| {
                format!("{APPLICATION_FIREWALL}: socketfilterfw printed no state this build reads")
            })
        }),
    }
}

fn pf_of(dump: &FirewallDump) -> Result<PfRead, String> {
    let enabled = parse_pf_info(dump.printed(PF_INFO)?)
        .ok_or_else(|| format!("{PF_INFO}: pfctl printed no status this build reads"))?;

    let main = PfRuleset {
        name: MAIN.to_string(),
        filter: parse_pf_rules(dump.printed(PF_RULES)?),
        translation: parse_pf_rules(dump.printed(PF_NAT)?),
    };

    let mut anchors = Vec::new();
    let mut anchors_unread = Vec::new();
    let listed = dump
        .printed(PF_ANCHORS)
        .map(parse_pf_anchors)
        .unwrap_or_default();
    for anchor in listed {
        let rules = dump.printed(&format!("{ANCHOR_RULES}{anchor}"));
        let nat = dump.printed(&format!("{ANCHOR_NAT}{anchor}"));
        match (rules, nat) {
            (Ok(rules), Ok(nat)) => anchors.push(PfRuleset {
                name: anchor,
                filter: parse_pf_rules(rules),
                translation: parse_pf_rules(nat),
            }),
            (Err(why), _) | (_, Err(why)) => anchors_unread.push(why),
        }
    }
    if let Err(why) = dump.printed(PF_ANCHORS) {
        anchors_unread.push(why);
    }

    Ok(PfRead {
        enabled,
        main,
        anchors,
        anchors_unread,
    })
}
