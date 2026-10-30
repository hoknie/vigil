use crate::finding::{Kind, KnownKind, RESERVED};

#[test]
fn a_reserved_kind_is_a_known_kind() {
    assert!(!RESERVED.is_empty());
    for reserved in RESERVED {
        assert!(
            KnownKind::ALL.contains(reserved),
            "{} is reserved and missing from the vocabulary",
            reserved.as_str()
        );
        assert_eq!(
            Kind::from(reserved.as_str().to_string()),
            Kind::Known(*reserved),
            "a receiver reading {} from another producer reads a kind it knows, not an unknown \
             string",
            reserved.as_str()
        );
    }
}
