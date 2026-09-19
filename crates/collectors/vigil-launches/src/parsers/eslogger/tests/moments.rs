use super::super::moment::epoch_of;

#[test]
fn a_moment_eslogger_writes_is_read_as_seconds_since_the_epoch_in_utc() {
    assert_eq!(epoch_of("1970-01-01T00:00:00Z"), Some((0, 0)));
    assert_eq!(
        epoch_of("2026-09-17T09:00:01.000Z"),
        Some((1_789_635_601, 0))
    );
    assert_eq!(
        epoch_of("2024-02-29T23:59:59.999999999Z"),
        Some((1_709_251_199, 999))
    );
}

#[test]
fn a_moment_this_parser_cannot_place_is_refused_rather_than_read_as_the_epoch() {
    for broken in [
        "",
        "2026-09-17",
        "2026-09-17T09:00:01",
        "2026-13-17T09:00:01Z",
        "2026-09-17T25:00:01Z",
        "2026-09-17T09:00:01.x1Z",
        "2026-09-17 09:00:01Z",
    ] {
        assert_eq!(epoch_of(broken), None, "{broken:?}");
    }
}
