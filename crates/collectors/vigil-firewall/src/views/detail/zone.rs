use vigil_view::Piece;

use crate::types::Zone;

const WHAT_A_ZONE_IS: &str = "nftables has no zones. firewalld builds one out of chains: a rule \
                              on a base chain sends a packet that arrived on one of these \
                              interfaces into the chains below, and what happens to it is \
                              decided there. This agent reads the chains and the jumps, not \
                              firewalld's own configuration, so what is named here is what the \
                              kernel is actually doing.";

pub(in crate::views) fn zone(zone: &Zone) -> Vec<Piece> {
    let mut said = vec![
        Piece::title("ZONE", zone.name.clone()),
        Piece::Blank,
        Piece::field("name", zone.name.clone()),
        Piece::field("reached from", zone.shown_on()),
        Piece::field("chains", zone.through.len().to_string()),
        Piece::Blank,
        Piece::heading("THE CHAINS A PACKET OF THIS ZONE IS SENT INTO"),
        Piece::Blank,
    ];

    for chain in &zone.through {
        said.push(Piece::line(format!("   {chain}")));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT THIS MEANS"));
    said.push(Piece::text(WHAT_A_ZONE_IS));
    said.push(Piece::Blank);

    said
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zone_says_which_interfaces_reach_it_and_which_chains_decide_what_happens_there() {
        let public = Zone {
            name: "public".to_string(),
            on: vec!["eth0".to_string()],
            through: vec!["filter_IN_public".to_string()],
        };

        let said = format!("{:?}", zone(&public));

        assert!(said.contains("public"), "{said}");
        assert!(said.contains("eth0"), "{said}");
        assert!(said.contains("filter_IN_public"), "{said}");
    }

    #[test]
    fn a_zone_says_where_this_build_read_it_from_because_it_is_not_a_thing_nftables_has() {
        let said = format!(
            "{:?}",
            zone(&Zone {
                name: "home".to_string(),
                on: Vec::new(),
                through: Vec::new(),
            })
        );

        assert!(
            said.contains("firewalld"),
            "an operator told that this host has a zone goes looking for it, and where they \
             look is firewall-cmd rather than nft: {said}"
        );
        assert!(said.contains("any interface"), "{said}");
    }
}
