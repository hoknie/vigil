use serde_json::json;

use crate::types::Family;
use crate::views::fields::kind_of_store;

#[test]
fn a_host_whose_swap_has_no_fixed_size_is_drawn_without_one_and_not_as_no_swap() {
    let without =
        json!({"total_bytes": 8_589_934_592u64, "swap_total_bytes": null, "readable": true});
    let with = json!({"total_bytes": 8_589_934_592u64, "swap_total_bytes": 0, "readable": true});

    assert_eq!(
        kind_of_store(Family::Memory, &without),
        "swap —",
        "macOS grows its swap as it goes and the reading leaves the size out, and a zero on \
         the screen would say the host has none"
    );
    assert_eq!(kind_of_store(Family::Memory, &with), "swap 0 B");
}
