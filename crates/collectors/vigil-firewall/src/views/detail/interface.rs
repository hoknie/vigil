use serde_json::Value;
use vigil_view::Piece;

use super::super::fields;

const NOT_COUNTED: &str = "What went through this interface is not in the reading. Counting it \
                           is off until monitoring is switched on for this host, because a \
                           counter moves every time a packet arrives and a reading carrying \
                           one differs from the reading before it on every single pass.";

const WHERE_IT_IS_READ: &str = "The interfaces are read from /proc/net: the names and counters \
                                from /proc/net/dev, this host's own addresses from \
                                /proc/net/fib_trie placed on the interface whose route holds \
                                them, and the sixth family from /proc/net/if_inet6.";

pub(super) fn interface(key: &str, item: &Value) -> Vec<Piece> {
    let mut said = vec![
        Piece::title("LINK", fields::what(key, item)),
        Piece::Blank,
        Piece::field("name", fields::what(key, item)),
        Piece::field("address", fields::shown_addresses(item)),
        Piece::field(
            "way out",
            match fields::the_way_out(item) {
                true => "yes",
                false => "no",
            },
        ),
        Piece::field("object", key),
        Piece::Blank,
    ];

    said.push(Piece::heading("WHAT WENT THROUGH IT"));
    match fields::counted(item) {
        false => said.push(Piece::text(NOT_COUNTED)),
        true => {
            said.push(Piece::Blank);
            for (name, value) in [
                ("in", counted(item, "packets_in", "bytes_in")),
                ("out", counted(item, "packets_out", "bytes_out")),
                (
                    "dropped",
                    format!(
                        "{} in, {} out",
                        fields::flow(item, "dropped_in"),
                        fields::flow(item, "dropped_out")
                    ),
                ),
            ] {
                said.push(Piece::field(name, value));
            }
        }
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHERE THIS COMES FROM"));
    said.push(Piece::text(WHERE_IT_IS_READ));
    said.push(Piece::Blank);

    said
}

fn counted(item: &Value, packets: &str, bytes: &str) -> String {
    format!(
        "{} packet(s), {}",
        fields::flow(item, packets),
        vigil_view::bytes(fields::flow(item, bytes))
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_interface_that_is_counted_says_what_came_in_what_went_out_and_what_was_dropped() {
        let watched = json!({
            "name": "eth0", "addresses": ["192.168.1.23"], "the_way_out": true, "counted": true,
            "packets_in": 9_112, "packets_out": 4_004, "bytes_in": 1_204_881,
            "bytes_out": 331_207, "dropped_in": 3, "dropped_out": 0
        });

        let said = format!("{:?}", interface("fw-interface|eth0", &watched));

        assert!(said.contains("9112 packet(s)"), "{said}");
        assert!(said.contains("4004 packet(s)"), "{said}");
        assert!(said.contains("3 in, 0 out"), "{said}");
    }

    #[test]
    fn an_interface_nobody_asked_to_count_says_why_there_is_no_number_rather_than_showing_zero() {
        let quiet = json!({
            "name": "eth0", "addresses": [], "the_way_out": false, "counted": false
        });

        let said = format!("{:?}", interface("fw-interface|eth0", &quiet));

        assert!(
            said.contains("is off until monitoring is switched on"),
            "{said}"
        );
        assert!(
            !said.contains("0 packet(s)"),
            "a zero written where a number nobody took would go reads as an interface nothing \
             passed through: {said}"
        );
        assert!(said.contains(fields::NO_ADDRESS), "{said}");
    }
}
