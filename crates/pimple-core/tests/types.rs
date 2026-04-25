use jiff::Zoned;
use jiff::civil::Date;
use pimple_core::{CollectionId, EventTime};

#[test]
fn collection_id_displays_as_inner_string() {
    let id = CollectionId::new("personal");
    assert_eq!(id.as_str(), "personal");
    assert_eq!(format!("{id}"), "personal");
}

#[test]
#[expect(clippy::expect_used, reason = "test — infallible with valid inputs")]
fn collection_id_round_trips_through_serde_json() {
    let id = CollectionId::new("work");
    let s = serde_json::to_string(&id).expect("CollectionId serializes");
    let back: CollectionId = serde_json::from_str(&s).expect("CollectionId deserializes");
    assert_eq!(id, back);
}

#[test]
#[expect(clippy::expect_used, reason = "test — infallible with valid inputs")]
fn event_time_all_day_serializes_as_date() {
    let et = EventTime::AllDay {
        date: Date::constant(2026, 4, 25),
    };
    let s = serde_json::to_string(&et).expect("EventTime::AllDay serializes");
    assert!(s.contains("\"all_day\""));
    assert!(s.contains("2026-04-25"));
}

#[test]
#[expect(clippy::expect_used, reason = "test — infallible with valid inputs")]
fn event_time_zoned_serializes_with_tz() {
    let z: Zoned = "2026-04-25T15:30:00[America/New_York]"
        .parse()
        .expect("valid zoned datetime literal");
    let et = EventTime::Zoned { zoned: z };
    let s = serde_json::to_string(&et).expect("EventTime::Zoned serializes");
    assert!(s.contains("America/New_York"));
}
