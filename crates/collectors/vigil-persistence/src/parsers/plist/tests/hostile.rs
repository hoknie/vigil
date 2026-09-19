use super::super::binary::MOST_OBJECTS_VISITED;
use super::super::{PlistRefusal, parse_plist};

const BINARY: &[u8] = include_bytes!("job.binary");

fn trailer(objects: u64, top: u64, table: u64) -> Vec<u8> {
    let mut trailer = vec![0u8; 6];
    trailer.push(1);
    trailer.push(1);
    trailer.extend_from_slice(&objects.to_be_bytes());
    trailer.extend_from_slice(&top.to_be_bytes());
    trailer.extend_from_slice(&table.to_be_bytes());
    trailer
}

#[test]
fn an_array_that_holds_itself_is_refused_rather_than_read_forever() {
    let mut bytes = b"bplist00".to_vec();
    bytes.extend_from_slice(&[0xA1, 0x00]);
    bytes.push(8);
    bytes.extend(trailer(1, 0, 10));

    assert_eq!(parse_plist(&bytes), Err(PlistRefusal::TooDeep));
}

#[test]
fn objects_referred_to_over_and_over_are_refused_before_they_are_counted_out() {
    let levels = 40u8;
    let mut bytes = b"bplist00".to_vec();
    let mut offsets = Vec::new();
    for level in 0..levels {
        offsets.push(bytes.len() as u8);
        match level + 1 < levels {
            true => bytes.extend_from_slice(&[0xA2, level + 1, level + 1]),
            false => bytes.push(0x09),
        }
    }
    let table = bytes.len() as u64;
    bytes.extend_from_slice(&offsets);
    bytes.extend(trailer(u64::from(levels), 0, table));

    match parse_plist(&bytes) {
        Err(PlistRefusal::Broken(why)) => assert!(why.contains("over and over"), "{why}"),
        other => panic!(
            "two references per level is 2^40 objects from a file of 200 bytes, and the cap \
             of {MOST_OBJECTS_VISITED} is what stops it: {other:?}"
        ),
    }
}

#[test]
fn a_binary_property_list_cut_anywhere_is_refused_and_never_read_past_its_end() {
    for length in [0, 8, 20, 39, BINARY.len() / 2, BINARY.len() - 1] {
        assert!(
            parse_plist(&BINARY[..length]).is_err(),
            "{length} bytes of {}",
            BINARY.len()
        );
    }
}

#[test]
fn a_trailer_that_points_outside_the_file_is_refused() {
    let mut bytes = BINARY.to_vec();
    let at = bytes.len() - 8;
    bytes[at..].copy_from_slice(&u64::MAX.to_be_bytes());

    assert!(matches!(parse_plist(&bytes), Err(PlistRefusal::Broken(_))));
}

#[test]
fn a_file_that_is_no_property_list_is_refused_as_what_it_is() {
    assert_eq!(parse_plist(b""), Err(PlistRefusal::Empty));
    assert_eq!(parse_plist(b" \n"), Err(PlistRefusal::Empty));
    assert_eq!(
        parse_plist(b"#!/bin/sh\nexec /tmp/x\n"),
        Err(PlistRefusal::NotAPropertyList)
    );
    assert!(matches!(
        parse_plist(b"<plist><dict><key>Label</key></dict></plist>"),
        Err(PlistRefusal::Broken(_))
    ));
    assert!(matches!(
        parse_plist(b"<plist><string>open</plist>"),
        Err(PlistRefusal::Broken(_))
    ));
}

#[test]
fn nesting_deeper_than_any_job_is_refused() {
    let deep = format!(
        "<plist>{}{}</plist>",
        "<array>".repeat(100),
        "</array>".repeat(100)
    );

    assert_eq!(parse_plist(deep.as_bytes()), Err(PlistRefusal::TooDeep));
}
