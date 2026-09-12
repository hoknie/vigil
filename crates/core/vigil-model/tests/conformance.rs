use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vigil_model::{Envelope, KnownKind, SCHEMA_VERSION, SchemaVersion, Severity};

fn contract() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/contract")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn manifest() -> Value {
    serde_json::from_str(&read(&contract().join("conformance/manifest.json"))).expect("manifest")
}

fn cases() -> Vec<Value> {
    manifest()["cases"]
        .as_array()
        .expect("cases is a list")
        .clone()
}

#[test]
fn every_golden_document_is_accepted_and_survives_a_round_trip() {
    let mut checked = 0;
    for case in cases() {
        if case["expect"] != "accept" {
            continue;
        }
        let file = contract()
            .join("conformance")
            .join(case["file"].as_str().expect("file"));
        let text = read(&file);

        let envelope: Envelope = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{} must parse: {error}", file.display()));
        assert!(
            SCHEMA_VERSION.accepts(envelope.schema_version),
            "{} carries {}, which this build refuses",
            file.display(),
            envelope.schema_version
        );

        let again: Value = serde_json::to_value(&envelope).expect("re-serialises");
        let original: Value = serde_json::from_str(&text).expect("re-reads");
        for finding in original["findings"].as_array().expect("findings") {
            let key = finding["event_id"].as_str().expect("event id");
            let ours = again["findings"]
                .as_array()
                .expect("findings")
                .iter()
                .find(|f| f["event_id"] == key)
                .unwrap_or_else(|| panic!("{} lost the finding {key}", file.display()));
            for (field, value) in finding.as_object().expect("object") {
                if field == "fields_from_the_future" {
                    continue;
                }
                assert_eq!(&ours[field], value, "{}: {field} changed", file.display());
            }
        }
        checked += 1;
    }
    assert!(
        checked >= 5,
        "the golden set has shrunk to {checked} documents"
    );
}

#[test]
fn a_kind_and_a_severity_this_build_never_heard_of_survive_instead_of_being_dropped() {
    let file = contract().join("conformance/accept/from-a-newer-producer.json");

    let envelope: Envelope = serde_json::from_str(&read(&file)).expect("parses");
    let finding = &envelope.findings[0];

    assert_eq!(finding.kind.as_str(), "port.listen.moved_to_wireguard");
    assert_eq!(finding.severity, Severity::Unknown("catastrophic".into()));
    let out = serde_json::to_value(finding).expect("re-serialises");
    assert_eq!(out["kind"], "port.listen.moved_to_wireguard");
    assert_eq!(out["severity"], "catastrophic");
}

#[test]
fn documents_that_must_be_refused_are_refused_by_the_stated_mechanism() {
    for case in cases() {
        if case["expect"] != "reject" {
            continue;
        }
        let name = case["file"].as_str().expect("file");
        let by = case["rejected_by"]
            .as_str()
            .expect("every rejection names its mechanism");
        let text = read(&contract().join("conformance").join(name));
        let parsed: Result<Envelope, _> = serde_json::from_str(&text);

        match by {
            "parser" => assert!(parsed.is_err(), "{name} was supposed to fail to parse"),
            "version" => {
                let envelope = parsed.unwrap_or_else(|e| panic!("{name} must parse first: {e}"));
                assert!(
                    !SCHEMA_VERSION.accepts(envelope.schema_version),
                    "{name} was supposed to be refused by the version rule"
                );
            }
            "schema" => {
                let envelope = parsed.unwrap_or_else(|e| panic!("{name} must still parse: {e}"));
                assert!(SCHEMA_VERSION.accepts(envelope.schema_version), "{name}");
            }
            other => panic!("{name}: unknown rejection mechanism {other:?}"),
        }
    }
}

#[test]
fn the_schema_carries_the_same_vocabulary_as_this_build() {
    let schema: Value =
        serde_json::from_str(&read(&contract().join("host-findings-v1.schema.json")))
            .expect("the schema is json");

    let listed: BTreeSet<String> = schema["$defs"]["finding"]["properties"]["kind"]["examples"]
        .as_array()
        .expect("the schema lists the kinds")
        .iter()
        .map(|value| value.as_str().expect("a kind is text").to_string())
        .collect();
    let known: BTreeSet<String> = KnownKind::ALL
        .iter()
        .map(|kind| kind.as_str().to_string())
        .collect();

    assert_eq!(
        listed, known,
        "the schema and the vocabulary have drifted — whichever was changed, change the other"
    );

    let severities: BTreeSet<String> =
        schema["$defs"]["finding"]["properties"]["severity"]["examples"]
            .as_array()
            .expect("the schema lists the severities")
            .iter()
            .map(|value| value.as_str().expect("a severity is text").to_string())
            .collect();
    let ours: BTreeSet<String> = Severity::KNOWN
        .iter()
        .map(|severity| severity.as_str().to_string())
        .collect();
    assert_eq!(severities, ours, "the severity vocabulary has drifted");
}

#[test]
fn the_schema_names_the_version_this_build_speaks() {
    let schema: Value =
        serde_json::from_str(&read(&contract().join("host-findings-v1.schema.json")))
            .expect("the schema is json");

    let id = schema["$id"].as_str().expect("an id");
    assert!(
        id.ends_with(&format!(
            "host-findings-v{}.schema.json",
            SCHEMA_VERSION.major
        )),
        "{id} does not name MAJOR {}",
        SCHEMA_VERSION.major
    );
    assert!(SCHEMA_VERSION.accepts(SchemaVersion {
        major: SCHEMA_VERSION.major,
        minor: 0
    }));
}
