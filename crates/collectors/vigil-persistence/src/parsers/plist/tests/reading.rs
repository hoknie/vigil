use super::super::{PlistValue, parse_plist};

const XML: &[u8] = include_bytes!("job.xml");

const BINARY: &[u8] = include_bytes!("job.binary");

fn without(value: PlistValue, dropped: &str) -> PlistValue {
    match value {
        PlistValue::Dictionary(mut entries) => {
            entries.remove(dropped);
            PlistValue::Dictionary(entries)
        }
        other => other,
    }
}

#[test]
fn a_job_written_as_xml_is_read_as_the_dictionary_launchd_reads() {
    let job = parse_plist(XML).expect("reads");

    assert_eq!(
        job.get("Label").and_then(PlistValue::as_str),
        Some("com.example.updater")
    );
    assert_eq!(
        job.get("ProgramArguments")
            .and_then(PlistValue::as_array)
            .map(<[PlistValue]>::len),
        Some(4)
    );
    assert_eq!(
        job.get("RunAtLoad").and_then(PlistValue::as_bool),
        Some(true)
    );
    assert_eq!(
        job.get("StartInterval").and_then(PlistValue::as_integer),
        Some(3600)
    );
    assert_eq!(job.get("ExitTimeOut"), Some(&PlistValue::Real(12.5)));
    assert_eq!(job.get("Seal"), Some(&PlistValue::Data(8)));
}

#[test]
fn the_same_job_written_as_a_binary_property_list_is_the_same_dictionary() {
    let from_xml = without(parse_plist(XML).expect("reads"), "Built");
    let from_binary = without(parse_plist(BINARY).expect("reads"), "Built");

    assert_eq!(
        from_binary, from_xml,
        "plutil converts one into the other, and launchd reads both"
    );
}

#[test]
fn text_is_read_with_its_entities_turned_back_into_characters() {
    let job = parse_plist(XML).expect("reads");

    assert_eq!(
        job.get("Comment").and_then(PlistValue::as_str),
        Some("Keeps \"Example\" & its helpers <current>!")
    );
}

#[test]
fn a_date_is_kept_as_written_and_a_binary_one_as_seconds_since_1970() {
    assert_eq!(
        parse_plist(XML).expect("reads").get("Built"),
        Some(&PlistValue::Date("2026-09-01T10:00:00Z".to_string()))
    );
    assert_eq!(
        parse_plist(BINARY).expect("reads").get("Built"),
        Some(&PlistValue::Date("1788256800".to_string()))
    );
}
